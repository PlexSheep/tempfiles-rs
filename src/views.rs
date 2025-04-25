use std::io::Read;
use std::path::PathBuf;

use actix_multipart::form::MultipartForm;
use actix_web::dev::AppService;
use actix_web::{App, HttpResponse, HttpServer, Responder, get, post, web};
use log::{debug, info, trace, warn};

use crate::errors::Error;
use crate::files::FileUpload;
use crate::state::AppState;

#[get("/")]
pub async fn view_index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/file")]
pub async fn view_post_file(
    state: web::Data<AppState>,
    MultipartForm(mut file_upload): MultipartForm<FileUpload>,
) -> Result<impl Responder, Error> {
    info!("File upload requested");
    debug!("file upload data: {file_upload:?}");

    let mut new_path = state.storage_dir();
    new_path.push(state.new_fid().await.to_string());
    std::fs::create_dir(&new_path)?;
    new_path.push(file_upload.name.to_string());
    std::fs::rename(file_upload.file.file.path(), &new_path).inspect_err(|e| {
        warn!("Error while uploading file: {e}");
        debug!(
            "Path of temporary upload was: {}",
            file_upload.file.file.path().display()
        )
    })?;

    Ok(HttpResponse::Ok().body(format!("uploaded to {}", new_path.display())))
}

#[get("/file/{fid}")]
pub async fn view_get_file_fid(data: web::Data<AppState>, req_body: String) -> impl Responder {
    todo!();
    HttpResponse::Ok().body("TODO")
}

#[get("/file/{fid}/{filename}")]
pub async fn view_get_file_fid_name(data: web::Data<AppState>, req_body: String) -> impl Responder {
    todo!();
    HttpResponse::Ok().body("TODO")
}
