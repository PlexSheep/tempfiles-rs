use std::fmt::Display;

use actix_identity::Identity;
use argon2::PasswordHash;
use argon2::PasswordHasher;
use chrono::NaiveDateTime;
use log::debug;
use log::error;
use log::warn;
use rand::distr::SampleString;
use sea_orm::ActiveModelTrait;
use sea_orm::ColumnTrait as _;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use sea_orm::ModelTrait;
use sea_orm::QueryFilter as _;
use serde::Deserialize;
use serde::Serialize;
use serde_repr::Deserialize_repr;
use serde_repr::Serialize_repr;
use validator::Validate;

use crate::db;
use crate::db::schema::user::Entity as UserEntity;
use crate::db::schema::user::Model as UserModel;
use crate::db::schema::user_token;
use crate::db::schema::user_token::Entity as UserTokenE;
use crate::db::schema::user_token::Model as UserTokenM;
use crate::errors::Error;

pub type UserID = i32;

pub const HASH_ENCODING: argon2::password_hash::Encoding = argon2::password_hash::Encoding::B64;
pub const APIV1_TOKEN_PREFIX: &str = "tfr_";
pub const APIV1_TOKEN_SECRET_LEN: usize = 36;

#[derive(Debug, Serialize, Clone)]
pub struct User {
    pub(crate) inner: UserModel,
}

#[derive(Debug, Deserialize, Clone, Validate)]
pub struct UserLoginDataWeb {
    #[validate(email)]
    email: String,
    #[validate(length(min = 10, max = 512))]
    password: String,
}

#[derive(Debug, Deserialize, Clone, Validate)]
pub struct UserLoginDataApiV1 {
    #[validate(length(min = 40, max = 512))]
    token: String,
}

#[derive(Debug, Deserialize, Clone)]
pub enum UserLoginData {
    Web(UserLoginDataWeb),
    ApiV1(UserLoginDataApiV1),
}

#[derive(Debug, Deserialize, Clone, Validate)]
pub struct UserRegisterData {
    #[validate(email)]
    email: String,
    #[validate(length(min = 10, max = 512))]
    password: String,
    #[validate(length(min = 3, max = 40))]
    username: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UserKind {
    Anonymous,
    #[default]
    Standard,
    Admin,
}

#[derive(Debug, Deserialize_repr, Serialize_repr, Clone)]
#[repr(u16)]
pub enum CredentialDuration {
    D30 = 30,
    D90 = 90,
    D365 = 365,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ApiV1TokenRequest {
    #[serde(rename = "tokenDuration")]
    pub requested_duration: CredentialDuration,
}

impl From<CredentialDuration> for chrono::TimeDelta {
    fn from(value: CredentialDuration) -> Self {
        chrono::TimeDelta::days(match value {
            CredentialDuration::D30 => 30,
            CredentialDuration::D90 => 90,
            CredentialDuration::D365 => 345,
        })
    }
}

impl UserLoginData {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        match self {
            Self::ApiV1(a) => a.validate(),
            Self::Web(a) => a.validate(),
        }
    }

    async fn find_user_with_db(&self, db: &DatabaseConnection) -> Result<Option<UserModel>, Error> {
        match self {
            Self::Web(a) => Ok(UserEntity::find()
                .filter(<UserEntity as EntityTrait>::Column::Email.eq(&a.email))
                .one(db)
                .await?),
            Self::ApiV1(login_data) => {
                let tokens = UserTokenE::find().all(db).await?;
                let now = chrono::Utc::now().naive_utc();

                for token_model in tokens {
                    if token_model.expiration_time < now {
                        #[cfg(debug_assertions)] // this should likely not be printed in production
                        warn!("APIV1 Token is expired: {}", token_model.token);
                        continue;
                    }

                    // stored token is valid
                    if token_model.token == login_data.token {
                        let user = token_model.find_related(UserEntity).one(db).await?;
                        return Ok(user);
                    }
                }
                Ok(None)
            }
        }
    }

    async fn authenticate(
        &self,
        user_model: &UserModel,
        db: &DatabaseConnection,
    ) -> Result<(), Error> {
        match self {
            Self::Web(a) => User::auth_with_password(user_model, a),
            Self::ApiV1(a) => User::auth_with_token_v1(user_model, a, db).await,
        }
    }
}

impl User {
    pub async fn get_by_id(id: UserID, db: &DatabaseConnection) -> Result<Self, Error> {
        let user = UserEntity::find_by_id(id).one(db).await?;
        if user.is_none() {
            return Err(Error::UserDoesNotExist);
        }
        let inner = user.unwrap();

        Ok(User { inner })
    }

    pub async fn login(login_data: UserLoginData, db: &DatabaseConnection) -> Result<Self, Error> {
        login_data.validate()?;

        let user = login_data.find_user_with_db(db).await?;

        if user.is_none() {
            return Err(Error::UserDoesNotExist);
        }
        let inner = user.unwrap();

        login_data.authenticate(&inner, db).await?;

        Ok(User { inner })
    }

    pub async fn register_and_insert(
        register_data: UserRegisterData,
        kind: UserKind,
        db: &DatabaseConnection,
        salt: argon2::password_hash::Salt<'_>,
    ) -> Result<Self, Error> {
        register_data.validate()?;

        let hashed_pass = Self::hash_password(&register_data.password, salt)?;

        let now_utc = chrono::Utc::now();
        let now: chrono::NaiveDateTime = now_utc.naive_utc();
        let new_user = db::schema::user::ActiveModel {
            email: sea_orm::ActiveValue::Set(register_data.email.clone()),
            password_hash: sea_orm::ActiveValue::Set(hashed_pass.to_string()),
            creation_time: sea_orm::ActiveValue::Set(now),
            last_action_time: sea_orm::ActiveValue::Set(now),
            user_name: sea_orm::ActiveValue::Set(register_data.username),
            kind: sea_orm::ActiveValue::Set(kind.to_string()),
            ..Default::default()
        };
        let _new_user: UserModel = new_user.insert(db).await?;

        Self::login(
            UserLoginData::Web(UserLoginDataWeb {
                email: register_data.email,
                password: register_data.password,
            }),
            db,
        )
        .await
    }

    pub async fn logout(self, _db: &DatabaseConnection) -> Result<(), Error> {
        Ok(()) // implicit drop of self
    }

    fn argon2<'t>() -> argon2::Argon2<'t> {
        argon2::Argon2::default()
    }

    fn load_password_hash(stored_hash: &str) -> Result<argon2::password_hash::PasswordHash, Error> {
        match PasswordHash::parse(stored_hash, HASH_ENCODING) {
            Ok(hash) => Ok(hash),
            Err(e) => {
                error!("Error while loading the password hash from the database: {e}");
                #[cfg(debug_assertions)]
                {
                    debug!("Hash that could not be loaded: {stored_hash}");
                }
                Err(Error::PwHashing(e))
            }
        }
    }

    fn hash_password<'t>(
        cleartext: &str,
        salt: argon2::password_hash::Salt<'t>,
    ) -> Result<argon2::password_hash::PasswordHash<'t>, Error> {
        let a = Self::argon2();
        let hash = match a.hash_password(cleartext.as_bytes(), salt) {
            Ok(h) => h,
            Err(e) => {
                error!("Error while hashing password: {e}");
                return Err(Error::PwHashing(e));
            }
        };

        Ok(hash)
    }

    pub fn username(&self) -> &str {
        &self.inner.user_name
    }

    pub fn email(&self) -> &str {
        &self.inner.email
    }

    pub fn id(&self) -> UserID {
        self.inner.id
    }

    fn auth_with_password(
        user_model: &UserModel,
        login_data: &UserLoginDataWeb,
    ) -> Result<(), Error> {
        let real_hash = Self::load_password_hash(&user_model.password_hash)?;
        let hashed_pass = Self::hash_password(
            &login_data.password,
            real_hash.salt.expect("password did not have a salt"),
        )?;
        if real_hash != hashed_pass {
            warn!("Bad login attempt for {}", login_data.email);
            Err(Error::WrongPassword)
        } else {
            Ok(())
        }
    }

    async fn auth_with_token_v1(
        user_model: &UserModel,
        login_data: &UserLoginDataApiV1,
        db: &DatabaseConnection,
    ) -> Result<(), Error> {
        let tokens = user_model.find_related(UserTokenE).all(db).await?;
        let mut authenticated = false;
        let now = chrono::Utc::now().naive_utc();

        for token_model in tokens {
            if token_model.expiration_time < now {
                #[cfg(debug_assertions)] // this should likely not be printed in production
                warn!("APIV1 Token is expired: {}", token_model.token);
                continue;
            }

            // stored token is valid
            if token_model.token == login_data.token {
                authenticated = true;
                break;
            }
        }

        if !authenticated {
            warn!("Bad login attempt for {}", user_model.email);
            Err(Error::WrongPassword)
        } else {
            Ok(())
        }
    }

    pub async fn create_api_v1_token(
        &self,
        expiration_time: NaiveDateTime,
        rng: &mut impl rand::CryptoRng,
        db: &DatabaseConnection,
    ) -> Result<UserTokenM, Error> {
        let token_secret: String =
            rand::distr::Alphanumeric.sample_string(rng, APIV1_TOKEN_SECRET_LEN);
        let token = format!("{APIV1_TOKEN_PREFIX}{token_secret}");
        let now = chrono::Utc::now().naive_utc();

        let token_model = user_token::ActiveModel {
            token: sea_orm::ActiveValue::Set(token),
            creation_time: sea_orm::ActiveValue::Set(now),
            expiration_time: sea_orm::ActiveValue::Set(expiration_time),
            user_id: sea_orm::ActiveValue::Set(self.id()),
        }
        .insert(db)
        .await?;

        Ok(token_model)
    }
}

impl Display for UserKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Admin => "administrator",
                Self::Standard => "standard",
                Self::Anonymous => "anonymous",
            }
        )
    }
}

pub async fn maybe_user(
    session_identity: &Option<Identity>,
    db: &DatabaseConnection,
) -> Result<Option<User>, Error> {
    let maybe_user = match session_identity {
        None => return Ok(None),
        Some(i) => get_user_from_identity(i, db).await,
    };
    match maybe_user {
        Ok(u) => Ok(Some(u)),
        Err(e) => match e {
            Error::UserDoesNotExist => Ok(None),
            other => Err(other),
        },
    }
}

pub async fn get_user_from_identity(
    session_identity: &Identity,
    db: &DatabaseConnection,
) -> Result<User, Error> {
    User::get_by_id(session_identity.id()?.parse()?, db).await
}
