use std::str::FromStr;

use actix_multipart::form::MultipartForm;
use actix_web::web::Redirect;
use actix_web::{HttpResponse, Responder, get, post, web};
use log::{debug, info, warn};

use crate::errors::Error;
use crate::files::{FileID, FileUpload};
use crate::state::AppState;

#[post("/file")]
pub async fn api_view_post_file(
    state: web::Data<AppState<'_>>,
    MultipartForm(file_upload): MultipartForm<FileUpload>,
) -> Result<impl Responder, Error> {
    info!("Uploading File");
    debug!("file upload data: {file_upload:?}");

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

    Ok(HttpResponse::Ok().json(state.make_file_infos(fid, &name).await?))
}

#[get("/file/{fid}")]
pub async fn api_view_get_file_fid(
    state: web::Data<AppState<'_>>,
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
    state: web::Data<AppState<'_>>,
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
    state: web::Data<AppState<'_>>,
    urlpath: web::Path<(String, String)>,
) -> Result<impl Responder, Error> {
    info!("Get information on file for fid");
    let urlargs = urlpath.into_inner();
    let fid = FileID::from_str(&urlargs.0)?;
    let name = urlargs.1;
    Ok(HttpResponse::Ok().json(state.make_file_infos(fid, &name).await?))
}
