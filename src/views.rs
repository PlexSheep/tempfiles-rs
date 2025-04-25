use std::io::Read;

use actix_multipart::form::MultipartForm;
use actix_web::dev::AppService;
use actix_web::{App, HttpResponse, HttpServer, Responder, get, post, web};
use log::{debug, info, trace};

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
) -> impl Responder {
    info!("File upload requested");
    debug!("file upload: {file_upload:?}");
    let mut content = String::new();
    file_upload.file.file.read_to_string(&mut content).unwrap();
    HttpResponse::Ok().body(content)
}

#[get("/file/{fid}")]
pub async fn view_get_file_fid(data: web::Data<AppState>, req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

#[get("/file/{fid}/{filename}")]
pub async fn view_get_file_fid_name(data: web::Data<AppState>, req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}
