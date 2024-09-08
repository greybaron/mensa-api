use anyhow::Result;
use chrono::{DateTime, Datelike, Duration, FixedOffset, NaiveDate, Weekday};
use tokio::task::JoinSet;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::{
    constants::{MENSEN_MAP, MENSEN_MAP_INV},
    stuwe_request_funcs::{invert_map, parse_and_save_meals},
};

pub async fn start_mensacache_and_campusdual_job() {
    let sched = JobScheduler::new().await.unwrap();

    let cache_job = Job::new_async("0 0/5 * * * *", move |_uuid, mut _l| {
        Box::pin(async move {
            log::info!("Updating Mensae");

            if let Err(e) = update_cache().await {
                println!("Failed to update cache: {}", e);
            }
        })
    })
    .unwrap();
    sched.add(cache_job).await.unwrap();
    sched.start().await.unwrap();
}

pub async fn update_cache() -> Result<()> {
    // will be run periodically: requests all mensa plans for the next 7 days
    // returns a vector of mensa locations whose 'today' plan was updated (here only used for dbg logging)

    let today = chrono::Local::now();
    let mut days: Vec<NaiveDate> = Vec::new();
    for i in 0..7 {
        let day: DateTime<FixedOffset> = (today + Duration::days(i)).into();

        if ![Weekday::Sat, Weekday::Sun].contains(&day.weekday()) {
            days.push(day.date_naive());
        }
    }

    // add tasks to joinset to execute concurrently
    let mut set = JoinSet::new();
    let mut mensen_today_changed = Vec::new();

    let mensen_map_inv_before = MENSEN_MAP_INV.read().unwrap().clone();

    for day in &days {
        set.spawn(parse_and_save_meals(*day));
    }

    while let Some(res) = set.join_next().await {
        match res? {
            Ok(mut changed_mensen_ids) => {
                mensen_today_changed.append(&mut changed_mensen_ids);
            }
            Err(e) => {
                log::warn!("Error in cache execution: {}", e);
            }
        }
    }

    let mensen_map_inv_now = MENSEN_MAP_INV.read().unwrap();
    if mensen_map_inv_before != *mensen_map_inv_now {
        *MENSEN_MAP.write().unwrap() = invert_map(&mensen_map_inv_now);
    }

    log::info!(
        "{} Mensen changed meals of current day",
        mensen_today_changed.len()
    );

    Ok(())
}
