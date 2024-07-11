use std::sync::Arc;

use axum::{
    extract::{Query, State},
    Json,
};
use chrono::NaiveDate;
use http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{
    db_operations::get_meals_from_db,
    types::{AppState, MealGroup, Mensa, ResponseError},
};
// use anyhow::Result;

#[derive(Serialize, Deserialize)]
struct UserResponse {
    test: String,
}

pub async fn get_mensa_list(State(state): State<Arc<AppState>>) -> Json<Vec<Mensa>> {
    let mut mensa_list: Vec<Mensa> = Vec::new();

    for (id, name) in state.data.lock().await.iter() {
        mensa_list.push(Mensa {
            id: *id,
            name: name.clone(),
        });
    }

    Json(mensa_list)
}

#[derive(Deserialize, Debug)]
pub struct MealsQuery {
    pub mensa: u8,
    pub date: String,
}

pub async fn get_day_at_mensa(
    State(state): State<Arc<AppState>>,
    params: Query<MealsQuery>,
) -> Result<Json<Vec<MealGroup>>, ResponseError> {
    let date = NaiveDate::parse_from_str(&params.date, "%Y-%m-%d");
    match date {
        Err(_) => Err(ResponseError {
            message: "Invalid date format".to_string(),
            status_code: StatusCode::BAD_REQUEST,
        }),
        Ok(date) => {
            if state.data.lock().await.get(&params.mensa).is_none() {
                return Err(ResponseError {
                    message: "Mensa not found".to_string(),
                    status_code: StatusCode::NOT_FOUND,
                });
            }
            let day_meals = get_meals_from_db(date, params.mensa).await.unwrap();
            Ok(Json(day_meals))
        }
    }
    // let date_str = "2021-10-01";
    // let date = chrono::DateTime::parse_from_rfc2822("2024-07-11").unwrap().naive_local();

    // dbg!(date);
    // let date = DateTime::naive_local(&self)
    // let day_meals = get_meals_from_db(date, 170).await.unwrap().unwrap();

    // Json(day_meals)
    // Ok(Json(vec![]))
}
