use std::{env, sync::Arc};
use tokio::{net::TcpListener, sync::Mutex};
use types::AppState;

mod cronjobs;
mod db_operations;
mod routes;
mod services;
mod stuwe_request_funcs;
mod types;
use cronjobs::{start_mensacache_and_campusdual_job, update_cache};
use db_operations::{check_or_create_db_tables, init_mensa_id_db};
use stuwe_request_funcs::get_mensen;

#[tokio::main]
async fn main() {
    if env::var(pretty_env_logger::env_logger::DEFAULT_FILTER_ENV).is_err() {
        env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init_timed();
    log::info!("Starting API...");

    //// DB setup
    check_or_create_db_tables().unwrap();

    let mensen = get_mensen().await.unwrap();
    init_mensa_id_db(&mensen).unwrap();

    let shared_state = Arc::new(AppState {
        data: Mutex::new(mensen),
    });

    // always update cache on startup
    match update_cache().await {
        Ok(_) => log::info!("Cache updated!"),
        Err(e) => log::error!("Cache update failed: {}", e),
    }

    start_mensacache_and_campusdual_job().await;

    let listener = TcpListener::bind("0.0.0.0:9090")
        .await
        .expect("Unable to conne to connect to the server");

    log::info!("Listening on {}", listener.local_addr().unwrap());

    let app = routes::app(shared_state).await;

    axum::serve(listener, app)
        .await
        .expect("Error serving application");
}
