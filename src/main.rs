use std::path::PathBuf;

use actix_identity::IdentityMiddleware;
use actix_multipart::form::MultipartFormConfig;
use actix_session::SessionMiddleware;
use actix_session::config::PersistentSession;
use actix_session::storage::CookieSessionStore;
use actix_web::cookie::Key;
use actix_web::cookie::time::Duration;
use actix_web::http::KeepAlive;
use actix_web::middleware::Logger;
use actix_web::web::{FormConfig, PayloadConfig};
use actix_web::{App, HttpServer, web};
use actix_web_static_files::ResourceFiles;
use garbage_collector::garbage_collector;
use log::trace;

mod api_v1;
mod auth;
mod config;
mod db;
mod errors;
mod files;
mod garbage_collector;
mod state;
mod urls;
mod user;
mod views;

use self::api_v1::*;
use self::config::actix_config_global;
use self::errors::Error;
use self::state::AppState;
use self::state::load_config;
use self::views::*;

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

#[actix_web::main]
async fn main() -> Result<(), Error> {
    setup_logging(None);

    let config = load_config("./data/config.toml")?;

    let inner_state = AppState::new(&config).await?;
    let app_state = web::Data::new(inner_state);
    let app_state_gc = app_state.clone();
    let largest_possible_upload = config.largest_possible_upload();

    tokio::spawn(async move { garbage_collector(app_state_gc).await });

    HttpServer::new(move || {
        let generated_static_files = generate();
        let session_key: Key = Key::derive_from(config.service.secret.clone().as_bytes());

        App::new()
            .configure(actix_config_global)
            .app_data(app_state.clone())
            .app_data(PayloadConfig::new(largest_possible_upload))
            .app_data(FormConfig::default().limit(largest_possible_upload))
            .app_data(MultipartFormConfig::default().total_limit(largest_possible_upload))
            .wrap(Logger::default())
            .wrap(IdentityMiddleware::default())
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), session_key)
                    .cookie_name("auth-tempfilesrs".to_owned())
                    .cookie_secure(false)
                    .session_lifecycle(
                        PersistentSession::default().session_ttl(Duration::hours(48)),
                    )
                    .build(),
            )
            .service(ResourceFiles::new("/static", generated_static_files))
            .service(frontend_view_get_index)
            .service(frontend_view_get_file_fid)
            .service(frontend_view_get_file_fid_name)
            .service(frontend_view_get_login)
            .service(frontend_view_get_register)
            .service(frontend_view_post_index)
            .service(frontend_view_post_login)
            .service(frontend_view_get_logout)
            .service(frontend_view_post_register)
            .service(frontend_view_get_settings)
            .service(frontend_view_get_about)
            .service(
                web::scope("/api/v1")
                    .service(api_view_get_file_fid_name)
                    .service(api_view_get_file_fid_name_info)
                    .service(api_view_get_file_fid)
                    .service(api_view_post_file)
                    .service(api_view_post_auth_token)
                    .service(api_view_get_auth_token)
                    .service(api_view_delete_auth_token_name),
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
    let ll_self = log::LevelFilter::Trace;
    env_logger::builder()
        .filter(Some(env!("CARGO_PKG_NAME")), ll_self)
        .filter(Some(env!("CARGO_BIN_NAME")), ll_self)
        .filter(Some("tempfiles_rs"), ll_self)
        .filter(Some("sqlx"), log::LevelFilter::Warn)
        .filter(None, log::LevelFilter::Debug)
        .try_init()
        .expect("could not set up logging");

    trace!("set up the logger");
}
