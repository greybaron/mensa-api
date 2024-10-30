use constants::{CANTEEN_MAP, CANTEEN_MAP_INV};
use openmensa_funcs::init_openmensa_canteenlist;
use std::env;
use tokio::{net::TcpListener, sync::broadcast};

mod constants;
mod cronjobs;
mod db_operations;
mod openmensa_funcs;
mod routes;
mod services;
mod stuwe_request_funcs;
mod types;
use cronjobs::{start_canteen_cache_job, update_cache};
use db_operations::{check_or_create_db_tables, get_canteens_from_db};
use stuwe_request_funcs::invert_map;

#[tokio::main]
async fn main() {
    if env::var(pretty_env_logger::env_logger::DEFAULT_FILTER_ENV).is_err() {
        env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init_timed();
    log::info!("Starting Mensa API...");

    //// DB setup
    check_or_create_db_tables().unwrap();

    {
        let canteens = get_canteens_from_db().await.unwrap();
        *CANTEEN_MAP_INV.write().unwrap() = invert_map(&canteens);
        *CANTEEN_MAP.write().unwrap() = canteens;
    }

    // stuwe_request_funcs::_run_benchmark().await.unwrap();
    // return;

    tokio::spawn(async {
        if let Err(e) = init_openmensa_canteenlist().await {
            log::error!("OpenMensa list fetch failed: {}", e);
        }
    });

    // always update cache on startup
    // dont pass 'today updated tx', this would cause erroneous WS broadcasts when cache
    // is too outdated or doesnt exist
    match update_cache(None).await {
        Ok(_) => log::info!("Cache updated!"),
        Err(e) => log::error!("Cache update failed: {}", e),
    }

    // set up broadcast channel to notify WS clients whenever today's canteen plans changed
    let (today_updated_tx, _) = broadcast::channel(20);

    start_canteen_cache_job(today_updated_tx.clone()).await;

    let listener = TcpListener::bind("0.0.0.0:9090")
        .await
        .expect("Unable to conne to connect to the server");

    log::info!("Listening on {}", listener.local_addr().unwrap());

    let app = routes::app(today_updated_tx).await;

    // used for building profiling data as i'm too lazy to set up test/bench
    // if env::var_os("PGOONLY").is_some() {
    //     for _ in 0..20 {
    //         update_cache().await.unwrap();
    //     }
    //     std::process::exit(0);
    // }

    axum::serve(listener, app)
        .await
        .expect("Error serving application");
}
