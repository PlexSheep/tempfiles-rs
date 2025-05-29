use actix_web::web::Data;
use log::{debug, error, info};
use sea_orm::ModelTrait;

use crate::{errors::Error, state::AppState as InnerAppState};

type AppState<'a> = Data<InnerAppState<'a>>;

pub async fn garbage_collector(state: AppState<'_>) {
    loop {
        info!("Running garbage collector workload");
        run_with_guard(async || clear_expired_files(state.clone()).await).await;

        info!("Workload finished, sleeping until next interval");
        tokio::time::sleep(state.garbage_collection_duration()).await;
    }
}

async fn clear_expired_files(state: AppState<'_>) -> Result<(), Error> {
    let now = chrono::Utc::now().naive_utc();
    for file in state.files().await? {
        if file.expiration_time < now {
            info!("File has expired: {}", file.id);
            let file_dir = state.upload_dir_for_fid(file.id, false)?;
            std::fs::remove_dir_all(file_dir)?;
            file.delete(state.db()).await?;
        } else {
            #[cfg(debug_assertions)]
            debug!("File is not expired but was checked: {}", file.id)
        }
    }
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
