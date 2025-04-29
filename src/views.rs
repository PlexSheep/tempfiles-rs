use std::str::FromStr;

use actix_identity::Identity;
use actix_web::web::Redirect;
use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder, get, post, web};
use argon2::password_hash::SaltString;
use minijinja::context;
use serde::Serialize;

use crate::user::{self, User, UserID, UserLoginData, UserRegisterData};

#[derive(Debug, Serialize)]
pub struct BasicContext {
    user: Option<user::User>,
}

impl BasicContext {
    pub async fn build(_state: &web::Data<AppState<'_>>) -> Result<Self, Error> {
        Ok(BasicContext { user: None })
    }
}

use crate::errors::Error;
use crate::files::FileID;
use crate::state::AppState;
#[get("/")]
pub async fn frontend_view_get_index(
    state: web::Data<AppState<'_>>,
    identity: Option<Identity>,
) -> Result<impl Responder, Error> {
    let content: String = state
        .templating()
        .get_template("index.html")?
        .render(context!(bctx => BasicContext::build(&state).await?))?;
    Ok(HttpResponse::Ok().body(content))
}

#[get("/file/{fid}")]
pub async fn frontend_view_get_file_fid(
    state: web::Data<AppState<'_>>,
    identity: Option<Identity>,
    path: web::Path<String>,
) -> Result<impl Responder, Error> {
    let fid: crate::files::FileID = FileID::from_str(&path.into_inner())?;
    let name = state.get_filename_for_fid(fid)?;

    Ok(Redirect::to(
        state.uri_frontend_file_fid_name(fid, &name).to_string(),
    ))
}

#[get("/file/{fid}/{name}")]
pub async fn frontend_view_get_file_fid_name(
    state: web::Data<AppState<'_>>,
    identity: Option<Identity>,
    urlpath: web::Path<(String, String)>,
) -> Result<impl Responder, Error> {
    let urlargs = urlpath.into_inner();
    let fid = FileID::from_str(&urlargs.0)?;
    let name = urlargs.1;
    let finfo = state.make_file_infos(fid, &name)?;
    let ct = finfo.content_type()?;

    let content: String = state
        .templating()
        .get_template("preview.html")?
        .render(context!(
            bctx => BasicContext::build(&state).await?,
            finfo => finfo,
            is_image => ct.type_() == mime::IMAGE
        ))?;
    Ok(HttpResponse::Ok().body(content))
}

pub async fn view_default() -> HttpResponse {
    HttpResponse::NotFound().body("No site for this URI")
}

#[get("/login")]
pub async fn frontend_view_get_login(
    state: web::Data<AppState<'_>>,
    identity: Option<Identity>,
) -> Result<impl Responder, Error> {
    let content: String = state
        .templating()
        .get_template("login.html")?
        .render(context!(bctx => BasicContext::build(&state).await?))?;
    Ok(HttpResponse::Ok().body(content))
}

#[post("/login")]
pub async fn frontend_view_post_login(
    req: HttpRequest,
    state: web::Data<AppState<'_>>,
    web::Form(login_data): web::Form<UserLoginData>,
) -> Result<impl Responder, Error> {
    let user = User::login(login_data, state.db()).await?;
    session_login(&req, &user)?;
    Ok(Redirect::to(state.uri_frontend_index().to_string()))
}

#[get("/register")]
pub async fn frontend_view_get_register(
    state: web::Data<AppState<'_>>,
    identity: Option<Identity>,
) -> Result<impl Responder, Error> {
    let content: String = state
        .templating()
        .get_template("register.html")?
        .render(context!(bctx => BasicContext::build(&state).await?))?;
    Ok(HttpResponse::Ok().body(content))
}

#[post("/register")]
pub async fn frontend_view_post_register(
    req: HttpRequest,
    state: web::Data<AppState<'_>>,
    web::Form(register_data): web::Form<UserRegisterData>,
) -> Result<impl Responder, Error> {
    // HACK: This should use the csprng of the AppState, but that seems incompatible somehow?
    let salt: SaltString = SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
    let user = User::register_and_insert(
        register_data,
        user::UserKind::Standard,
        state.db(),
        salt.as_salt(),
    )
    .await?;

    session_login(&req, &user)?;
    Ok(Redirect::to(state.uri_frontend_index().to_string()))
}

#[get("/settings")]
pub async fn frontend_view_get_settings(
    state: web::Data<AppState<'_>>,
    identity: Identity,
) -> Result<impl Responder, Error> {
    let content: String = state
        .templating()
        .get_template("login.html")?
        .render(context!(bctx => BasicContext::build(&state).await?))?;
    Ok(HttpResponse::Ok().body(content))
}

fn session_login(req: &HttpRequest, user: &User) -> Result<(), Error> {
    Identity::login(&req.extensions(), user.id().to_string())?;

    Ok(())
}

fn session_logout(session_identity: Identity) -> Result<(), Error> {
    session_identity.logout();
    Ok(())
}
