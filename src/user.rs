use serde::Serialize;

use crate::db;
use crate::errors::Error;
use crate::db::schema::user::Model as UserModel;

#[derive(Debug, Serialize)]
pub struct User {
    inner: UserModel,
}

impl User {
    pub fn login(email: &str, password: &str) -> Result<Option<Self>, Error> {

    }

    pub fn logout(self) -> Result<(),Error> {
        Ok(()) // implicit drop of self
    }
}
