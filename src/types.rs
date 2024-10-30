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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CanteenMealDiff {
    pub canteen_id: u32,
    pub new_meals: Option<Vec<MealGroup>>,
    pub modified_meals: Option<Vec<MealGroup>>,
    pub modified_meals_ignoring_allergens: Option<Vec<MealGroup>>,
    pub removed_meals: Option<Vec<MealGroup>>,
}

pub trait HasChanges {
    fn has_changes(&self) -> bool;
}

impl HasChanges for CanteenMealDiff {
    fn has_changes(&self) -> bool {
        self.new_meals.is_some() || self.modified_meals.is_some() || self.removed_meals.is_some()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MealGroup {
    pub meal_type: String,
    pub sub_meals: Vec<SingleMeal>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SingleMeal {
    pub name: String,
    pub additional_ingredients: Vec<String>,
    pub allergens: Option<String>,
    pub variations: Option<Vec<MealVariation>>,
    pub price: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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
