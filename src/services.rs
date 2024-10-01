use axum::{extract::Path, Json};
use chrono::NaiveDate;
use http::StatusCode;

use crate::{
    constants::CANTEEN_MAP,
    db_operations::{get_meals_from_db, list_available_days_db},
    types::{Canteen, MealGroup, ResponseError},
};

pub async fn get_canteens() -> Json<Vec<Canteen>> {
    let mut canteen_list: Vec<Canteen> = Vec::new();

    for (id, name) in CANTEEN_MAP.read().unwrap().iter() {
        canteen_list.push(Canteen {
            id: *id,
            name: name.clone(),
        });
    }

    Json(canteen_list)
}

pub async fn get_canteen_meta(Path(canteen_id): Path<u32>) -> Result<Json<Canteen>, StatusCode> {
    match CANTEEN_MAP.read().unwrap().get(&canteen_id) {
        Some(name) => Ok(Json(Canteen {
            id: canteen_id,
            name: name.clone(),
        })),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn get_canteen_available_days(
    Path(canteen_id): Path<u32>,
) -> Result<Json<Vec<String>>, StatusCode> {
    if get_canteen_meta(Path(canteen_id)).await.is_err() {
        return Err(StatusCode::NOT_FOUND);
    };

    let available_days = list_available_days_db(canteen_id).unwrap_or_default();
    Ok(Json(available_days))
}

pub async fn get_meals_of_day(
    Path((canteen_id, date)): Path<(u32, String)>,
) -> Result<Json<Vec<MealGroup>>, ResponseError> {
    let date = NaiveDate::parse_from_str(&date, "%Y-%m-%d");
    match date {
        Err(_) => Err(ResponseError {
            message: "Invalid date format".to_string(),
            status_code: StatusCode::BAD_REQUEST,
        }),
        Ok(date) => {
            if CANTEEN_MAP.read().unwrap().get(&canteen_id).is_none() {
                return Err(ResponseError {
                    message: "Canteen not found".to_string(),
                    status_code: StatusCode::NOT_FOUND,
                });
            }
            let day_meals = get_meals_from_db(canteen_id, date).await.unwrap();
            Ok(Json(day_meals))
        }
    }
}
