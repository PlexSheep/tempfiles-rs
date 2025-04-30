use std::fmt::Display;

use argon2::PasswordHash;
use argon2::PasswordHasher;
use log::debug;
use log::error;
use log::warn;
use sea_orm::ActiveModelTrait;
use sea_orm::ColumnTrait;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use serde::Deserialize;
use serde::Serialize;
use validator::Validate;

use crate::db;
use crate::db::schema::user::Entity as UserEntity;
use crate::db::schema::user::Model as UserModel;
use crate::errors::Error;

pub type UserID = i32;

pub const HASH_ENCODING: argon2::password_hash::Encoding = argon2::password_hash::Encoding::Bcrypt;

#[derive(Debug, Serialize, Clone)]
pub struct User {
    pub(crate) inner: UserModel,
}

#[derive(Debug, Deserialize, Clone, Validate)]
pub struct UserLoginData {
    #[validate(email)]
    email: String,
    #[validate(length(min = 10, max = 512))]
    password: String,
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

        let user = UserEntity::find()
            .filter(<UserEntity as EntityTrait>::Column::Email.eq(&login_data.email))
            .one(db)
            .await?;
        if user.is_none() {
            return Err(Error::UserDoesNotExist);
        }
        let inner = user.unwrap();

        let real_hash = Self::load_password_hash(&inner.password_hash)?;
        let hashed_pass = Self::hash_password(
            &login_data.password,
            real_hash.salt.expect("password did not have a salt"),
        )?;
        if real_hash != hashed_pass {
            warn!("Bad login attempt for {}", login_data.email);
            return Err(Error::WrongPassword);
        }

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
            UserLoginData {
                email: register_data.email,
                password: register_data.password,
            },
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
                return Err(Error::PwHashing(e));
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
