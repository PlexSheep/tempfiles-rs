use argon2::PasswordHash;
use argon2::PasswordHasher;
use log::warn;
use sea_orm::ColumnTrait;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use serde::Serialize;

use crate::db::schema::user::Entity as UserEntity;
use crate::db::schema::user::Model as UserModel;
use crate::errors::Error;

type UserID = i32;

pub const HASH_ENCODING: argon2::password_hash::Encoding = argon2::password_hash::Encoding::Bcrypt;

#[derive(Debug, Serialize)]
pub struct User {
    inner: UserModel,
}

impl User {
    pub async fn login(
        email: &str,
        password: &str,
        db: &DatabaseConnection,
    ) -> Result<Self, Error> {
        let user = UserEntity::find()
            .filter(<UserEntity as EntityTrait>::Column::Email.eq(email))
            .one(db)
            .await?;
        if user.is_none() {
            return Err(Error::UserDoesNotExist);
        }
        let inner = user.unwrap();

        let real_hash = Self::load_password_hash(&inner.password_hash)?;
        let hashed_pass = Self::hash_password(
            password,
            real_hash.salt.expect("password did not have a salt"),
        )?;
        if real_hash != hashed_pass {
            warn!("Bad login attempt for {email}");
            return Err(Error::WrongPassword);
        }

        Ok(User { inner })
    }

    pub async fn register(
        email: &str,
        password: &str,
        db: &DatabaseConnection,
    ) -> Result<Self, Error> {
        todo!("validate email");
        todo!("validate password reqs (just len)");
        todo!("hash password");
        todo!("create and insert");
        todo!("automatic login");
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
            Err(e) => Err(Error::PwHashing(e)),
        }
    }

    fn hash_password<'t>(
        cleartext: &str,
        salt: argon2::password_hash::Salt<'t>,
    ) -> Result<argon2::password_hash::PasswordHash<'t>, Error> {
        let a = Self::argon2();
        let hash = match a.hash_password(cleartext.as_bytes(), salt) {
            Ok(h) => h,
            Err(e) => return Err(Error::PwHashing(e)),
        };

        Ok(hash)
    }
}
