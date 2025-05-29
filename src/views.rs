use std::io::Read;
use std::str::FromStr;

use actix_identity::Identity;
use actix_web::web::Redirect;
use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder, get, post, web};
use argon2::password_hash::SaltString;
use minijinja::context;
use serde::Serialize;

use crate::auth::{AuthUser, MaybeAuthUser};
use crate::errors::Error;
use crate::files::FileID;
use crate::state::AppState;
use crate::user::{self, User, UserLoginData, UserRegisterData};

#[derive(Debug, Serialize)]
pub struct BasicContext {
    user: Option<user::User>,
    base_url: String,
}

impl BasicContext {
    pub async fn build(
        state: &web::Data<AppState<'_>>,
        user: Option<user::User>,
    ) -> Result<Self, Error> {
        Ok(BasicContext {
            user,
            base_url: state.config().service.base_url.clone(),
        })
    }
}

async fn frontend_view_inner_index(
    state: web::Data<AppState<'_>>,
    identity: MaybeAuthUser,
) -> Result<impl Responder, Error> {
    let user: Option<User> = identity.user();

    let content: String = state
        .templating()
        .get_template("index.html")?
        .render(context!(bctx => BasicContext::build(&state, user).await?))?;
    Ok(HttpResponse::Ok().body(content))
}

#[get("/")]
pub async fn frontend_view_get_index(
    state: web::Data<AppState<'_>>,
    identity: MaybeAuthUser,
) -> Result<impl Responder, Error> {
    frontend_view_inner_index(state, identity).await
}

#[post("/")]
pub async fn frontend_view_post_index(
    state: web::Data<AppState<'_>>,
    identity: MaybeAuthUser,
) -> Result<impl Responder, Error> {
    frontend_view_inner_index(state, identity).await
}

#[get("/file/{fid}")]
pub async fn frontend_view_get_file_fid(
    state: web::Data<AppState<'_>>,
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
    identity: MaybeAuthUser,
    urlpath: web::Path<(String, String)>,
) -> Result<impl Responder, Error> {
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

    let content: String = state
        .templating()
        .get_template("preview.html")?
        .render(context!(
            bctx => BasicContext::build(&state, user).await?,
            finfo => finfo,
            content_type_general => ct.type_().to_string(),
            content_type_full => ct.to_string(),
            file_content => text_content
        ))?;
    Ok(HttpResponse::Ok().body(content))
}

pub async fn view_default() -> HttpResponse {
    HttpResponse::NotFound().body("No site for this URI")
}

#[get("/login")]
pub async fn frontend_view_get_login(
    state: web::Data<AppState<'_>>,
    identity: MaybeAuthUser,
) -> Result<impl Responder, Error> {
    let user = identity.user();

    let content: String = state
        .templating()
        .get_template("login.html")?
        .render(context!(bctx => BasicContext::build(&state, user).await?))?;
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

#[get("/logout")]
pub async fn frontend_view_get_logout(
    state: web::Data<AppState<'_>>,
    identity: MaybeAuthUser,
) -> Result<impl Responder, Error> {
    let user = identity.inner();
    if user.as_ref().is_some_and(|u| u.identity_ref().is_some()) {
        let user = user.unwrap();
        session_logout(user.identity().unwrap())?;
    }
    Ok(Redirect::to(state.uri_frontend_index().to_string()))
}

#[get("/register")]
pub async fn frontend_view_get_register(
    state: web::Data<AppState<'_>>,
    identity: MaybeAuthUser,
) -> Result<impl Responder, Error> {
    let user = identity.user();

    let content: String = state
        .templating()
        .get_template("register.html")?
        .render(context!(bctx => BasicContext::build(&state, user).await?))?;
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
    identity: AuthUser,
) -> Result<impl Responder, Error> {
    let user = identity.user();

    let content: String = state
        .templating()
        .get_template("settings.html")?
        .render(context!(bctx => BasicContext::build(&state, Some(user)).await?))?;
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
