use actix_web::{HttpResponse, Responder, get};
#[get("/")]
pub async fn view_get_index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}
