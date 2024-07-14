use anyhow::Result;
use chrono::{DateTime, Datelike, Duration, FixedOffset, NaiveDate, Weekday};
use tokio::task::JoinSet;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::campus_request_funcs::parse_and_save_meals;

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
}

pub async fn update_cache() -> Result<()> {
    // will be run periodically: requests all possible dates (heute/morgen/ueb) and creates/updates caches
    // returns a vector of mensa locations whose 'today' plan was updated

    // days will be selected using this rule:
    // if current day ... then ...

    //     Thu =>
    //         'heute' => thursday
    //         'morgen' => friday
    //         'uebermorgen' => monday

    //     Fri =>
    //         'heute' => friday
    //         'morgen'/'uebermorgen' => monday

    //     Sat =>
    //         'heute'/'morgen'/'uebermorgen' => monday

    //     Sun =>
    //         'heute'/'morgen' => monday
    //         'uebermorgen' => tuesday

    //     Mon/Tue/Wed => as you'd expect

    let mut days: Vec<NaiveDate> = Vec::new();

    // get at most 3 days from (inclusive) today, according to the rule above
    let today = chrono::Local::now();

    for i in 0..3 {
        let day: DateTime<FixedOffset> = (today + Duration::days(i)).into();

        if ![Weekday::Sat, Weekday::Sun].contains(&day.weekday()) {
            days.push(day.date_naive());
        // weekend day is not first day (i>0) but within 3 day range, so add monday & break
        } else if i != 0 {
            if day.weekday() == Weekday::Sat {
                days.push((day + Duration::days(2)).date_naive());
            } else {
                days.push((day + Duration::days(1)).date_naive());
            }
            break;
        }
    }

    // add tasks to joinset to execute concurrently
    let mut set = JoinSet::new();
    let mut mensen_today_changed = Vec::new();

    for day in &days {
        set.spawn(parse_and_save_meals(*day));
    }

    while let Some(res) = set.join_next().await {
        let mut changed_mensen_ids = res??;
        mensen_today_changed.append(&mut changed_mensen_ids);
    }

    log::info!(
        "{} Mensen changed meals of current day",
        mensen_today_changed.len()
    );

    Ok(())
}
