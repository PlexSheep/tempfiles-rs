use serde::Serialize;

use crate::db;
use crate::errors::Error;

#[derive(Debug, Serialize)]
pub struct User {
    inner: db::schema::user::Model,
}

impl User {
    pub fn login(email: &str, password: &str) -> Result<Option<Self>, Error> {
        todo!()
    }

    pub fn logout(self) -> Result<(),Error> {
        todo!()
    }
}
