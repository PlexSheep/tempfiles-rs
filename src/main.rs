use std::path::PathBuf;

use actix_web::http::KeepAlive;
use actix_web::middleware::Logger;
use actix_web::{App, HttpServer, web};
use env_logger::Env;
use log::trace;

mod api_v1;
mod config;
mod db;
mod errors;
mod files;
mod state;
mod urls;
mod user;
mod views;

use self::api_v1::*;
use self::config::actix_config_global;
use self::errors::Error;
use self::state::AppState;
use self::state::load_config;
use self::views::{
    frontend_view_get_file_fid, frontend_view_get_file_fid_name, frontend_view_get_index,
    frontend_view_get_login, frontend_view_get_register, frontend_view_get_settings, view_default,
};

#[actix_web::main]
async fn main() -> Result<(), Error> {
    setup_logging(None);

    let config = load_config("./data/config.toml")?;

    let inner_state = AppState::new(&config).await?;
    let app_state = web::Data::new(inner_state);

    HttpServer::new(move || {
        App::new()
            .configure(actix_config_global)
            .app_data(app_state.clone())
            .wrap(Logger::default())
            .service(frontend_view_get_index)
            .service(frontend_view_get_file_fid)
            .service(frontend_view_get_file_fid_name)
            .service(frontend_view_get_login)
            .service(frontend_view_get_register)
            .service(frontend_view_get_settings)
            .service(
                web::scope("/api/v1")
                    .service(api_view_get_file_fid_name)
                    .service(api_view_get_file_fid_name_info)
                    .service(api_view_get_file_fid)
                    .service(api_view_post_file),
            )
            .default_service(web::route().to(view_default))
    })
    .keep_alive(KeepAlive::Os) // TODO: check how long this is on debian
    .bind(&config.service.bind)? // TODO: use rustls
    .shutdown_timeout(15)
    .run()
    .await?;

    Ok(())
}

fn setup_logging(_logfile: Option<PathBuf>) {
    env_logger::init_from_env(Env::default().default_filter_or("debug"));

    trace!("set up the logger");
}
