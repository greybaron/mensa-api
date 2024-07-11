use std::sync::Arc;

use axum::{response::IntoResponse, routing::get, Router};
use http::{header::CONTENT_TYPE, Method};
use tower_http::cors::{Any, CorsLayer};

use crate::{services, types::AppState};

pub async fn app(shared_state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST])
        // allow requests from any origin
        .allow_origin(Any)
        .allow_headers([CONTENT_TYPE]);

    Router::new()
        .route("/", get(|| async { "API is reachable".into_response() }))
        .route("/mensalist", get(services::get_mensa_list))
        .with_state(shared_state.clone())
        .route("/get_day_at_mensa", get(services::get_day_at_mensa))
        .with_state(shared_state.clone())
        .layer(cors)
}
