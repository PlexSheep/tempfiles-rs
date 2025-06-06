use std::io::Read;
use std::str::FromStr;

use actix_identity::Identity;
use actix_web::body::BoxBody;
use actix_web::web::Either;
use actix_web::web::Redirect;
use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder, ResponseError, get, post, web};
use argon2::password_hash::SaltString;
use chrono::Datelike;
use log::trace;
use minijinja::context;
use serde::Serialize;

use crate::auth::{AuthUser, MaybeAuthUser};
use crate::config::Config;
use crate::db::schema::user_token::Model as UserTokenM;
use crate::errors::{Error, ErrorPageDetails};
use crate::files::FileID;
use crate::state::AppState;
use crate::user::{self, User, UserLoginData, UserLoginDataWeb, UserRegisterData};

#[derive(Debug, Serialize)]
pub struct BasicContext {
    user: Option<user::User>,
    base_url: String,
    version: String,
    pkgname: String,
    authors: String,
    homepage: String,
    copyright: String,
    config: Config,
    next_created_user_will_be_admin: bool,
}

impl BasicContext {
    pub async fn build(
        state: &web::Data<AppState>,
        user: Option<user::User>,
    ) -> Result<Self, Error> {
        let authors = env!("CARGO_PKG_AUTHORS").to_string();
        let now = chrono::Utc::now();
        Ok(BasicContext {
            user,
            base_url: state.config().service.base_url.clone(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            pkgname: env!("CARGO_PKG_NAME").to_string(),
            homepage: env!("CARGO_PKG_HOMEPAGE").to_string(),
            copyright: format!(
                "Copyright © {} {authors}.\nSome Rights Reserved.",
                now.year()
            ),
            authors,
            config: state.config().clone(),
            next_created_user_will_be_admin: state.next_created_user_will_be_admin().await?,
        })
    }
}
macro_rules! handle_frontend_error_inner {
    ($state:expr,$user:expr,$body:expr) => {{
            trace!("Starting frontend handler body...");
            let _body_f = async ||{$body};
            match _body_f().await {
                Ok(result) => {
                    trace!("Frontend Processed ok: {result:?}");
                    Ok(Either::Right(result)
                )},
                Err(e) => {
                    trace!("handling frontend error: {e:?}");
                    let status_code = e.status_code();
                    let error_details = ErrorPageDetails::from(e);
                    let content: String = $state.templating()?.get_template("error.html")?.render(
                        context!(bctx => BasicContext::build(&$state, $user).await?, error => error_details),
                    )?;
                    Ok(Either::Left(HttpResponse::Ok().status(status_code).body(content)))
                }
            }
    }};
}
macro_rules! handle_frontend_error {
    ($state:expr,Some($identity:expr),$body:expr) => {{
        let _user_for_error_handler: Option<User> = Some($identity);
        handle_frontend_error_inner!($state, _user_for_error_handler, $body)
    }};
    ($state:tt,$identity:expr,$body:expr) => {{
        let _user_for_error_handler = $identity.user_ref().map(|u| u.to_owned());
        handle_frontend_error_inner!($state, _user_for_error_handler, $body)
    }};
}

macro_rules! ok {
    ($return_type:ty,$stuff:expr) => {
        Ok::<$return_type, Error>($stuff)
    };
    ($stuff:expr) => {
        Ok::<HttpResponse, Error>($stuff)
    };
}

async fn frontend_view_inner_index(
    state: &web::Data<AppState>,
    identity: MaybeAuthUser,
) -> Result<HttpResponse<BoxBody>, Error> {
    let user: Option<User> = identity.user();

    let content: String = state
        .templating()?
        .get_template("index.html")?
        .render(context!(bctx => BasicContext::build(state, user).await?))?;
    Ok(HttpResponse::Ok().body(content))
}

async fn frontend_view_error(
    state: web::Data<AppState>,
    error: Error,
    identity: MaybeAuthUser,
) -> Result<HttpResponse<BoxBody>, Error> {
    let user = identity.user();
    let error_details = ErrorPageDetails::from(error);
    let content: String = state.templating()?.get_template("error.html")?.render(
        context!(bctx => BasicContext::build(&state, user).await?, error => error_details),
    )?;
    Ok(HttpResponse::Ok().body(content))
}

#[get("/")]
pub async fn frontend_view_get_index(
    state: web::Data<AppState>,
    identity: MaybeAuthUser,
) -> Result<impl Responder, Error> {
    handle_frontend_error!(
        state,
        identity,
        frontend_view_inner_index(&state, identity).await
    )
}

#[post("/")]
pub async fn frontend_view_post_index(
    state: web::Data<AppState>,
    identity: MaybeAuthUser,
) -> Result<impl Responder, Error> {
    handle_frontend_error!(state, identity, {
        frontend_view_inner_index(&state, identity).await
    })
}

#[post("/")]
pub async fn t(
    state: web::Data<AppState>,
    identity: MaybeAuthUser,
) -> Result<impl Responder, Error> {
    handle_frontend_error!(state, identity, {
        frontend_view_inner_index(&state, identity).await
    })
}

#[get("/about")]
pub async fn frontend_view_get_about(
    state: web::Data<AppState>,
    identity: MaybeAuthUser,
) -> Result<impl Responder, Error> {
    handle_frontend_error!(state, identity, {
        let user: Option<User> = identity.user();
        let content: String = state
            .templating()?
            .get_template("about.html")?
            .render(context!(bctx => BasicContext::build(&state, user).await?))?;
        ok!(HttpResponse::Ok().body(content))
    })
}

#[get("/file/{fid}")]
pub async fn frontend_view_get_file_fid(
    state: web::Data<AppState>,
    path: web::Path<String>,
    identity: MaybeAuthUser,
) -> Result<impl Responder, Error> {
    handle_frontend_error!(state, identity, {
        let fid: crate::files::FileID = FileID::from_str(&path.into_inner())?;
        let name = state.get_filename_for_fid(fid)?;

        trace!("done with body");
        ok!(
            Redirect,
            Redirect::to(state.uri_frontend_file_fid_name(fid, &name).to_string(),)
        )
    })
}

#[get("/file/{fid}/{name}")]
pub async fn frontend_view_get_file_fid_name(
    state: web::Data<AppState>,
    identity: MaybeAuthUser,
    urlpath: web::Path<(String, String)>,
) -> Result<impl Responder, Error> {
    handle_frontend_error!(state, identity, {
        let user = identity.user();

        let urlargs = urlpath.into_inner();
        let fid = FileID::from_str(&urlargs.0)?;
        let name = urlargs.1;
        let finfo = state.make_file_infos(fid, &name).await?;
        let ct = finfo.content_type()?;

        const MAX_PREVIEW_LENGTH: usize = 16384;
        let mut file_content_preview = vec![0; MAX_PREVIEW_LENGTH];
        let mut file = std::fs::File::open(state.upload_datafile_for_fid(fid, &name, false)?)?;
        #[allow(clippy::unused_io_amount)] // TODO: maybe need to handle partial reads
        file.read(&mut file_content_preview)?;
        let mut text_content: String = String::from_utf8_lossy(&file_content_preview).to_string();
        if file_content_preview.len() < finfo.size as usize {
            text_content.push_str("\n===============\n(abbreviated)");
        }
        let owns_this_file = user.is_some() && finfo.uploader.is_some() && user == finfo.uploader;

        let content: String =
            state
                .templating()?
                .get_template("preview.html")?
                .render(context!(
                    bctx => BasicContext::build(&state, user).await?,
                    finfo => finfo,
                    content_type_general => ct.type_().to_string(),
                    content_type_full => ct.to_string(),
                    file_content => text_content,
                    owns_this_file => owns_this_file,
                ))?;
        ok!(HttpResponse::Ok().body(content))
    })
}

pub async fn view_default(
    state: web::Data<AppState>,
    identity: MaybeAuthUser,
) -> actix_web::HttpResponse {
    let f = async move || {
        let user = identity.user();
        let e = Error::SiteDoesNotExist;
        let status_code = e.status_code();
        let error_details = ErrorPageDetails::from(e);
        let content: String = state.templating()?.get_template("error.html")?.render(
            context!(bctx => BasicContext::build(&state, user).await?, error => error_details),
        )?;
        ok!(HttpResponse::Ok().status(status_code).body(content))
    };
    f().await.expect("could not make default page")
}

#[get("/login")]
pub async fn frontend_view_get_login(
    state: web::Data<AppState>,
    identity: MaybeAuthUser,
) -> Result<impl Responder, Error> {
    handle_frontend_error!(state, identity, {
        let user = identity.user();

        let content: String = state
            .templating()?
            .get_template("login.html")?
            .render(context!(bctx => BasicContext::build(&state, user).await?))?;
        ok!(HttpResponse::Ok().body(content))
    })
}

#[post("/login")]
pub async fn frontend_view_post_login(
    req: HttpRequest,
    state: web::Data<AppState>,
    web::Form(login_data): web::Form<UserLoginDataWeb>,
    identity: MaybeAuthUser,
) -> Result<impl Responder, Error> {
    handle_frontend_error!(state, identity, {
        let user = User::login(UserLoginData::Web(login_data), state.db()).await?;
        session_login(&req, &user)?;
        ok!(
            Redirect,
            Redirect::to(state.uri_frontend_index().to_string())
        )
    })
}

#[get("/logout")]
pub async fn frontend_view_get_logout(
    state: web::Data<AppState>,
    identity: MaybeAuthUser,
) -> Result<impl Responder, Error> {
    handle_frontend_error!(state, identity, {
        let user = identity.inner();
        if user.as_ref().is_some_and(|u| u.identity_ref().is_some()) {
            let user = user.unwrap();
            session_logout(user.identity().unwrap())?;
        }
        ok!(
            Redirect,
            Redirect::to(state.uri_frontend_index().to_string())
        )
    })
}

#[get("/register")]
pub async fn frontend_view_get_register(
    state: web::Data<AppState>,
    identity: MaybeAuthUser,
) -> Result<impl Responder, Error> {
    handle_frontend_error!(state, identity, {
        let user = identity.user();

        let content: String = state
            .templating()?
            .get_template("register.html")?
            .render(context!(bctx => BasicContext::build(&state, user).await?))?;
        ok!(HttpResponse::Ok().body(content))
    })
}

#[post("/register")]
pub async fn frontend_view_post_register(
    req: HttpRequest,
    state: web::Data<AppState>,
    web::Form(register_data): web::Form<UserRegisterData>,
    identity: MaybeAuthUser,
) -> Result<impl Responder, Error> {
    handle_frontend_error!(state, identity, {
        let will_be_admin = state.next_created_user_will_be_admin().await?;
        if !state.config().accounts.allow_registration && !will_be_admin {
            return Err(Error::RegistrationClosed);
        }

        // HACK: This should use the csprng of the AppState, but there are two different versions of
        // the crate defining this trait in the dependencies
        let salt: SaltString = SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
        let user = User::register_and_insert(
            register_data,
            if will_be_admin {
                user::UserKind::Admin
            } else {
                user::UserKind::Standard
            },
            state.db(),
            salt.as_salt(),
        )
        .await?;

        session_login(&req, &user)?;
        ok!(
            Redirect,
            Redirect::to(state.uri_frontend_index().to_string())
        )
    })
}

#[get("/settings")]
pub async fn frontend_view_get_settings(
    state: web::Data<AppState>,
    identity: AuthUser,
) -> Result<impl Responder, Error> {
    let user = identity.user();
    handle_frontend_error!(state, Some(user.clone()), {
        let tokens: Vec<UserTokenM> = user.tokens(state.db()).await?;

        let content: String = state.templating()?.get_template("settings.html")?.render(
            context!(bctx => BasicContext::build(&state, Some(user)).await?, tokens => tokens),
        )?;
        ok!(HttpResponse::Ok().body(content))
    })
}

fn session_login(req: &HttpRequest, user: &User) -> Result<(), Error> {
    Identity::login(&req.extensions(), user.id().to_string())?;

    Ok(())
}

fn session_logout(session_identity: Identity) -> Result<(), Error> {
    session_identity.logout();
    Ok(())
}
