use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::{HttpResponse, Responder, get, web};
use minijinja::context;

use crate::errors::Error;
use crate::state::AppState;
#[get("/")]
pub async fn view_get_index(state: web::Data<AppState<'_>>) -> Result<impl Responder, Error> {
    let content: String = state
        .templating()
        .get_template("index.html")?
        .render(context!(test => "test ok"))?;
    Ok(HttpResponse::Ok().body(content))
}

pub async fn view_default() -> HttpResponse {
    HttpResponse::NotFound().body("No site for this URI")
}
