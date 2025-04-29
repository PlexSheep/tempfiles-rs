use std::str::FromStr;

use actix_web::web::Redirect;
use actix_web::{HttpResponse, Responder, get, web};
use minijinja::context;
use serde::Serialize;

use crate::user;

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
