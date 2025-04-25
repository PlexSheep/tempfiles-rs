use std::path::PathBuf;
use std::str::FromStr;

use actix_multipart::form::MultipartForm;
use actix_web::web::Redirect;
use actix_web::{HttpResponse, Responder, get, post, web};
use log::{debug, error, info, warn};

use crate::errors::Error;
use crate::files::{FileID, FileUpload};
use crate::state::AppState;

#[get("/")]
pub async fn view_get_index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/file")]
pub async fn view_post_file(
    state: web::Data<AppState>,
    MultipartForm(file_upload): MultipartForm<FileUpload>,
) -> Result<impl Responder, Error> {
    info!("File upload requested");
    debug!("file upload data: {file_upload:?}");

    let fid = state.new_fid().await;
    let mut new_path = state.upload_dir_for_fid(fid).await?;
    let name = file_upload.name.to_string().trim().to_string();
    new_path.push(&name);

    debug!(
        "Path of temporary upload: {}",
        file_upload.file.file.path().display()
    );
    info!("Uploading file to: {}", new_path.display());

    std::fs::rename(file_upload.file.file.path(), &new_path).inspect_err(|e| {
        warn!("Error while uploading file: {e}");
    })?;

    Ok(HttpResponse::Ok().body(format!(
        "uploaded to {}",
        state.url_for_fid_with_name(fid, &name)
    )))
}

#[get("/file/{fid}")]
pub async fn view_get_file_fid(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<impl Responder, Error> {
    let fid: crate::files::FileID = FileID::from_str(&path.into_inner())?;
    let path: PathBuf = state.upload_dir_for_fid(fid).await?;
    debug!("fid path: {}", path.display());
    if !path.exists() {
        debug!("does not exist");
        return Err(Error::FileNotFound);
    }

    let count_items = path.read_dir().into_iter().count();
    if count_items != 1 {
        error!("items in the directory: {count_items:?}");
        return Err(Error::NotOneFileInStorageDir(count_items));
    }
    let mut dir_ents: std::fs::ReadDir = path.read_dir()?;

    let item: Result<_, std::io::Error> = dir_ents
        .next()
        .expect("No dirent despite count_items being 1");
    if let Err(e) = item {
        return Err(e.into());
    }
    let name = item
        .as_ref()
        .unwrap()
        .file_name()
        .to_string_lossy()
        .to_string();
    debug!("name of file in fid path: {name}");

    Ok(Redirect::to(
        state.url_for_fid_with_name(fid, &name).to_string(),
    ))
}

#[get("/file/{fid}/{filename}")]
pub async fn view_get_file_fid_name(
    _state: web::Data<AppState>,
    _path: web::Path<(String, String)>,
) -> Result<impl Responder, Error> {
    Ok(HttpResponse::Ok().body("TODO"))
}

pub fn view_default() -> impl Responder {
    HttpResponse::NotFound().body("this page does not exist")
}
