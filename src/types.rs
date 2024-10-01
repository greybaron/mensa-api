use axum::{
    body::Body,
    response::{IntoResponse, Response},
};
use http::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Canteen {
    pub id: u32,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CanteenMealsDay {
    pub canteen_id: u32,
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

// API Response type
pub struct ResponseError {
    pub message: String,
    pub status_code: StatusCode,
}

impl IntoResponse for ResponseError {
    fn into_response(self) -> Response<Body> {
        (self.status_code, self.message).into_response()
    }
}
