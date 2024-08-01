use std::collections::BTreeMap;

use axum::{
    body::Body,
    response::{IntoResponse, Response},
    Json,
};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Deserialize)]
pub struct Mensa {
    pub id: u8,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DataForMensaForDay {
    pub mensa_id: u8,
    pub meal_groups: Vec<MealGroup>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MealGroup {
    pub meal_type: String,
    pub sub_meals: Vec<SingleMeal>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct SingleMeal {
    pub name: String,
    pub additional_ingredients: Vec<String>,
    pub allergens: Option<String>,
    pub variations: Option<Vec<MealVariation>>,
    pub price: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct MealVariation {
    pub name: String,
    pub allergens_and_add: Option<String>,
}

pub const DB_FILENAME: &str = "meals.sqlite";

pub struct AppState {
    pub data: Mutex<BTreeMap<u8, String>>,
}

// API Response type
pub struct ResponseError {
    pub message: String,
    pub status_code: StatusCode,
}

impl IntoResponse for ResponseError {
    fn into_response(self) -> Response<Body> {
        let body = Json(json!({
            "error": self.message,
        }));

        (self.status_code, body).into_response()
    }
}

impl From<anyhow::Error> for ResponseError {
    fn from(_: anyhow::Error) -> Self {
        ResponseError {
            message: "Internal Server Error".to_string(),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<reqwest::Error> for ResponseError {
    fn from(_: reqwest::Error) -> Self {
        ResponseError {
            message: "CampusDual is not reachable".to_string(),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
