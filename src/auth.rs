use actix_identity::Identity;
use actix_web::{Error, FromRequest, HttpRequest, Result, error::ErrorUnauthorized};
use futures_util::future::LocalBoxFuture;

use crate::state::AppState;
use crate::user::{User, UserLoginData, UserLoginDataApiV1, get_user_from_identity};

/// Authenticated user that can be extracted from either session cookies or API tokens
pub struct AuthUser(User, Option<Identity>);

impl AuthUser {
    pub fn user(self) -> User {
        self.0
    }

    pub fn user_ref(&self) -> &User {
        &self.0
    }

    pub fn identity(self) -> Option<Identity> {
        self.1
    }

    pub fn identity_ref(&self) -> Option<&Identity> {
        self.1.as_ref()
    }
}

impl std::ops::Deref for AuthUser {
    type Target = User;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequest for AuthUser {
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        let req = req.clone();

        Box::pin(async move {
            // Try API token first
            if let Some(auth_header) = req.headers().get("Authorization") {
                if let Ok(auth_str) = auth_header.to_str() {
                    if let Some(token) = auth_str.strip_prefix("Bearer ") {
                        // Get database connection from app state
                        if let Some(state) = req.app_data::<actix_web::web::Data<AppState>>() {
                            let login_data = UserLoginData::ApiV1(UserLoginDataApiV1 {
                                token: token.to_string(),
                            });

                            // Use existing authentication logic
                            match User::login(login_data, state.db()).await {
                                Ok(user) => return Ok(AuthUser(user, None)),
                                Err(e) => match e {
                                    crate::errors::Error::WrongPassword => {
                                        return Err(ErrorUnauthorized("Invalid Credentials"));
                                    }
                                    _other => (),
                                },
                            }
                        }
                    }
                }
            }

            // Fall back to session-based authentication
            if let Ok(identity) =
                Identity::from_request(&req, &mut actix_web::dev::Payload::None).into_inner()
            {
                if let Some(state) = req.app_data::<actix_web::web::Data<AppState>>() {
                    match get_user_from_identity(&identity, state.db()).await {
                        Ok(user) => return Ok(AuthUser(user, Some(identity))),
                        Err(_) => {
                            // Session authentication failed
                        }
                    }
                }
            }

            Err(ErrorUnauthorized("Authentication required"))
        })
    }
}

/// Optional authenticated user - returns None instead of error if not authenticated
pub struct MaybeAuthUser(Option<AuthUser>);

impl MaybeAuthUser {
    pub fn inner(self) -> Option<AuthUser> {
        self.0
    }

    pub fn inner_ref(&self) -> Option<&AuthUser> {
        self.0.as_ref()
    }

    pub fn user(self) -> Option<User> {
        self.inner().map(|u| u.user())
    }

    pub fn user_ref(&self) -> Option<&User> {
        self.inner_ref().map(|u| u.user_ref())
    }
}

impl std::ops::Deref for MaybeAuthUser {
    type Target = Option<AuthUser>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequest for MaybeAuthUser {
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut actix_web::dev::Payload) -> Self::Future {
        let auth_future = AuthUser::from_request(req, payload);

        Box::pin(async move {
            match auth_future.await {
                Ok(auth_user) => Ok(MaybeAuthUser(Some(auth_user))),
                Err(_) => Ok(MaybeAuthUser(None)),
            }
        })
    }
}
