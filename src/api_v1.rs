use std::ops::Deref;
use std::str::FromStr;

use actix_multipart::form::MultipartForm;
use actix_web::web::Redirect;
use actix_web::{HttpRequest, HttpResponse, Responder, delete, get, post, web};
use log::{debug, info, warn};
use sea_orm::ModelTrait;
use serde::{Serialize, Serializer};
use serde_json::json;

use crate::auth::{AuthUser, MaybeAuthUser};
use crate::errors::Error;
use crate::files::{FileID, FileUpload};
use crate::state::AppState;
use crate::user::{ApiV1TokenRequest, User};

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    #[serde(serialize_with = "ser_error")]
    error: crate::errors::Error,
}

#[post("/file")]
pub async fn api_view_post_file(
    state: web::Data<AppState>,
    MultipartForm(file_upload): MultipartForm<FileUpload>,
    identity: MaybeAuthUser,
) -> Result<impl Responder, Error> {
    let user = identity.user();
    let max_size = state.max_upload_size(user.as_ref())?;

    if user.is_none() && !state.config().accounts.allow_anon {
        return Ok(HttpResponse::Unauthorized()
            .json(json!({"error": format!("Only logged in users can upload to this instance")})));
    }

    info!("Uploading File");
    debug!("file upload data: {file_upload:?}");

    if file_upload.file.size as u64 > max_size {
        warn!("Uploaded file is too large: {}", file_upload.file.size);
        return Ok(HttpResponse::PayloadTooLarge()
            .json(json!({"error": format!("Uploaded file is too large: {max_size}")})));
    }

    let fid = state.new_fid().await;
    let name = file_upload
        .file
        .file_name
        .expect("why does it not have a file name???") // TODO: this will crash when no filename exists
        .to_string()
        .trim()
        .to_string();
    let new_path = state.upload_datafile_for_fid(fid, &name, true)?;

    debug!(
        "Path of temporary upload: {}",
        file_upload.file.file.path().display()
    );
    info!("Uploading file to: {}", new_path.display());

    std::fs::rename(file_upload.file.file.path(), &new_path).inspect_err(|e| {
        warn!("Error while uploading file: {e}");
    })?;

    state
        .create_file_db_entry(fid, user.as_ref(), state.db())
        .await?;

    Ok(HttpResponse::Ok().json(state.make_file_infos(fid, &name).await?))
}

#[get("/file/{fid}")]
pub async fn api_view_get_file_fid(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<impl Responder, Error> {
    let fid: crate::files::FileID = FileID::from_str(&path.into_inner())?;
    let name = state.get_filename_for_fid(fid)?;

    Ok(Redirect::to(
        state.uri_api_file_fid_name(fid, &name).to_string(),
    ))
}

#[get("/file/{fid}/{filename}")]
pub async fn api_view_get_file_fid_name(
    req: actix_web::HttpRequest,
    state: web::Data<AppState>,
    urlpath: web::Path<(String, String)>,
) -> Result<impl Responder, Error> {
    info!("Downloading file for fid");
    let urlargs = urlpath.into_inner();
    let fid = FileID::from_str(&urlargs.0)?;
    let name = urlencoding::decode(urlargs.1.as_str())?;
    let path = state.upload_datafile_for_fid(fid, &name, false)?;
    debug!("Get file: {}", path.display());
    let file = actix_files::NamedFile::open_async(&path).await?;
    Ok(file.into_response(&req))
}

#[get("/file/{fid}/{filename}/info")]
pub async fn api_view_get_file_fid_name_info(
    state: web::Data<AppState>,
    urlpath: web::Path<(String, String)>,
) -> Result<impl Responder, Error> {
    info!("Get information on file for fid");
    let urlargs = urlpath.into_inner();
    let fid = FileID::from_str(&urlargs.0)?;
    let name = urlargs.1;
    let finfos = state.make_file_infos(fid, &name).await?;
    debug!("finfos: {finfos:#?}");
    Ok(HttpResponse::Ok().json(finfos))
}

#[post("/auth/token")]
pub async fn api_view_post_auth_token(
    state: web::Data<AppState>,
    web::Form(token_request): web::Form<ApiV1TokenRequest>,
    identity: AuthUser,
) -> Result<impl Responder, Error> {
    let user: &User = identity.deref();

    info!("Creating new token for user: {}", user.email());

    let (token, token_data) = user
        .create_api_v1_token(token_request, &mut state.csprng().await, state.db())
        .await?;

    Ok(HttpResponse::Ok().json(json!({
        "token": &token,
        "time_expiration": &token_data.expiration_time,
        "time_creation": &token_data.creation_time
    })))
}

#[get("/auth/token")]
pub async fn api_view_get_auth_token(identity: AuthUser) -> Result<impl Responder, Error> {
    let user: &User = identity.deref();

    Ok(HttpResponse::Ok().json(json!({
        "authenticated": true,
        "email": user.email(),
        "name": user.username(),
        "id": user.id(),
        "userKind": user.kind()?,
    })))
}

#[delete("/auth/token/{token_name}")]
pub async fn api_view_delete_auth_token_name(
    state: web::Data<AppState>,
    identity: AuthUser,
    token_name: web::Path<String>,
) -> Result<impl Responder, Error> {
    let user: &User = identity.deref();
    let tokens = user.tokens(state.db()).await?;
    let token_name = token_name.into_inner();

    let mut found = None;
    for token in tokens {
        if token.name == token_name {
            found = Some(token.clone());
            token.delete(state.db()).await?;
        }
    }

    Ok(HttpResponse::Ok().json(json!({
        "deleted": found,
    })))
}

fn ser_error<S>(err: &crate::Error, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    err.to_string().serialize(s)
}
