use std::path::PathBuf;

use actix_web::http::KeepAlive;
use actix_web::middleware::Logger;
use actix_web::{App, HttpServer, web};
use env_logger::Env;
use log::trace;

mod config;
mod errors;
mod files;
mod state;
mod views;

use config::actix_config_global;
use errors::Error;
use state::AppState;
use views::*;

#[actix_web::main]
async fn main() -> Result<(), Error> {
    setup_logging(None);

    let inner_state = AppState::new("./conf/config.toml").await?;
    let app_state = web::Data::new(inner_state);

    HttpServer::new(move || {
        App::new()
            .configure(actix_config_global)
            .app_data(app_state.clone())
            .wrap(Logger::default())
            .service(view_index)
            .service(view_post_file)
    })
    .keep_alive(KeepAlive::Os) // TODO: check how long this is on debian
    .bind(("127.0.0.1", 8080))? // TODO: use rustls
    .shutdown_timeout(15)
    .run()
    .await?;

    Ok(())
}

fn setup_logging(_logfile: Option<PathBuf>) {
    env_logger::init_from_env(Env::default().default_filter_or("debug"));

    trace!("set up the logger");
}
