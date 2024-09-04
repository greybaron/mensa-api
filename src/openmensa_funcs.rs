use std::vec;

use anyhow::Result;
use axum::Json;
use http::StatusCode;
use reqwest::Client;
use reqwest_middleware::ClientBuilder;
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use serde::Deserialize;

use crate::{
    constants::{OPENMENSA_ALL_MENSEN, OPENMENSA_LIVE_MENSEN},
    types::Mensa,
};

#[derive(Deserialize)]
struct Day {
    closed: bool,
}

pub async fn init_openmensa_mensen_with_data() -> Result<()> {
    if OPENMENSA_ALL_MENSEN.get().is_some() {
        log::info!("OpenMensa list already initialized");
        return Ok(());
    }

    log::info!("Getting OpenMensa live mensen list, this might take a while");
    let mut mensen_with_days = vec![];

    let reqwest_client = Client::builder().build().unwrap();
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
    let client = ClientBuilder::new(reqwest_client)
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();

    let mensen: Vec<Mensa> = client
        .get("https://openmensa.org/api/v2/canteens")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    println!("got {} mensen", mensen.len());
    OPENMENSA_ALL_MENSEN.set(mensen.clone()).unwrap();

    for (iteration, mensa) in mensen.iter().enumerate() {
        if iteration % 50 == 0 {
            log::info!("{}%...", (iteration * 100) / mensen.len());
        }

        let days: Vec<Day> = client
            .get(format!(
                "https://openmensa.org/api/v2/canteens/{}/days",
                mensa.id
            ))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        if !days.is_empty() && !days.iter().all(|day| day.closed) {
            mensen_with_days.push(mensa.clone());
        }
    }

    OPENMENSA_LIVE_MENSEN.set(mensen_with_days).unwrap();
    Ok(())
}

pub async fn get_openmensa_list() -> Result<Json<Vec<Mensa>>, StatusCode> {
    if OPENMENSA_ALL_MENSEN.get().is_none() {
        return Err(StatusCode::NOT_FOUND);
    }
    if let Some(list) = OPENMENSA_LIVE_MENSEN.get() {
        Ok(Json(list.clone()))
    } else if let Some(list) = OPENMENSA_ALL_MENSEN.get() {
        log::warn!("OpenMensa live list not initialized, returning all mensen list");
        Ok(Json(list.clone()))
    } else {
        log::error!("OpenMensa list not initialized");
        Ok(Json(vec![]))
    }
}
