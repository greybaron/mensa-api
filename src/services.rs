use axum::{
    extract::{ws::WebSocket, Path, WebSocketUpgrade},
    response::IntoResponse,
    Json,
};
use chrono::NaiveDate;
use http::StatusCode;
use tokio::sync::broadcast;

use crate::{
    constants::CANTEEN_MAP,
    db_operations::{get_meals_from_db, list_available_days_db},
    types::{Canteen, MealGroup, ResponseError},
};

// handler to upgrade http to websocket connection
pub async fn ws_handler_today_upd(
    ws: WebSocketUpgrade,
    today_updated_tx: broadcast::Sender<String>,
) -> impl IntoResponse {
    // Upgrades the connection to a WebSocket and calls the `websocket` function to handle the connection.
    log::info!("WebSocket client connected");
    ws.on_upgrade(|socket| websocket_today_upd(socket, today_updated_tx))
}

// actual websocket handler after http->ws upgrade
// broadcasts a mensa id whenever its today's menu is updated
async fn websocket_today_upd(mut socket: WebSocket, today_updated_tx: broadcast::Sender<String>) {
    // each websocket instance has its own receiver
    let mut rx = today_updated_tx.subscribe();

    while let Ok(msg) = rx.recv().await {
        if socket
            .send(axum::extract::ws::Message::Text(msg))
            .await
            .is_err()
        {
            // client has disconnected
            break;
        }
    }
}

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
