use std::path::PathBuf;

use axum::{Router, routing::get};
use tracing::{info, trace};

#[tokio::main]
async fn main() {
    setup_logging(None);
    // build our application with a single route
    let app = Router::new().route("/", get(|| async { "Hello, World!" }));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("Starting app");
    axum::serve(listener, app).await.unwrap();
}

fn setup_logging(logfile: Option<PathBuf>) {
    if let Some(lf) = logfile {
        let file = match std::fs::File::options().create(true).append(true).open(lf) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("could not setup logfile: {e}");
                std::process::exit(1);
            }
        };

        // construct a subscriber that prints formatted traces to file
        let subscriber = tracing_subscriber::fmt()
            .with_max_level(
                #[cfg(debug_assertions)]
                tracing::Level::TRACE,
                #[cfg(not(debug_assertions))]
                tracing::Level::INFO,
            )
            .without_time()
            .with_file(false)
            .with_target(false)
            .with_writer(file)
            .finish();
        tracing::subscriber::set_global_default(subscriber).expect("could not setup logger");
    } else {
        let subscriber = tracing_subscriber::fmt()
            .with_max_level(
                #[cfg(debug_assertions)]
                tracing::Level::TRACE,
                #[cfg(not(debug_assertions))]
                tracing::Level::INFO,
            )
            .without_time()
            .with_file(false)
            .with_target(false)
            .with_writer(std::io::stderr)
            .finish();
        tracing::subscriber::set_global_default(subscriber).expect("could not setup logger");
    }

    trace!("set up the logger");
}
