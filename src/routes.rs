use axum::{response::IntoResponse, routing::get, Router};
use http::{header::CONTENT_TYPE, Method};
use tower_http::cors::{Any, CorsLayer};

use crate::services;

pub async fn app() -> Router {
    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST])
        // allow requests from any origin
        .allow_origin(Any)
        .allow_headers([CONTENT_TYPE]);

    Router::new()
        .route("/", get(|| async { "API is reachable".into_response() }))
        .route("/mensalist", get(services::get_mensa_list))
        .route("/get_day_at_mensa", get(services::get_day_at_mensa))
        .layer(cors)
}
