use axum::{extract::Query, Json};
use chrono::NaiveDate;
use http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{
    constants::MENSEN_MAP,
    db_operations::get_meals_from_db,
    types::{MealGroup, Mensa, ResponseError},
};
// use anyhow::Result;

#[derive(Serialize, Deserialize)]
struct UserResponse {
    test: String,
}

pub async fn get_mensa_list() -> Json<Vec<Mensa>> {
    let mut mensa_list: Vec<Mensa> = Vec::new();

    for (id, name) in MENSEN_MAP.get().unwrap() {
        mensa_list.push(Mensa {
            id: *id,
            name: name.clone(),
        });
    }

    Json(mensa_list)
}

#[derive(Deserialize, Debug)]
pub struct MealsQuery {
    pub mensa: u32,
    pub date: String,
}

pub async fn get_day_at_mensa(
    params: Query<MealsQuery>,
) -> Result<Json<Vec<MealGroup>>, ResponseError> {
    let date = NaiveDate::parse_from_str(&params.date, "%Y-%m-%d");
    match date {
        Err(_) => Err(ResponseError {
            message: "Invalid date format".to_string(),
            status_code: StatusCode::BAD_REQUEST,
        }),
        Ok(date) => {
            if MENSEN_MAP.get().unwrap().get(&params.mensa).is_none() {
                return Err(ResponseError {
                    message: "Mensa not found".to_string(),
                    status_code: StatusCode::NOT_FOUND,
                });
            }
            let day_meals = get_meals_from_db(date, params.mensa).await.unwrap();
            Ok(Json(day_meals))
        }
    }
}
