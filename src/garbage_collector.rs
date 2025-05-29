use actix_web::web::Data;
use log::{error, info};

use crate::{errors::Error, state::AppState as InnerAppState};

type AppState<'a> = Data<InnerAppState<'a>>;

pub async fn garbage_collector(state: AppState<'_>) {
    loop {
        info!("Running garbage collector workload");
        run_with_guard(async || clear_expired_files(state.clone()).await).await;

        info!("Workload finished, sleeping until next interval");
        tokio::time::sleep(state.garbage_collection_duraiton()).await;
    }
}

async fn clear_expired_files(_state: AppState<'_>) -> Result<(), Error> {
    Ok(())
}

async fn run_with_guard<F, T>(f: F) -> Option<T>
where
    F: AsyncFnOnce() -> Result<T, Error>,
{
    match f().await {
        Ok(t) => Some(t),
        Err(e) => {
            error!("Error while running garbage collection: {e}");
            None
        }
    }
}
