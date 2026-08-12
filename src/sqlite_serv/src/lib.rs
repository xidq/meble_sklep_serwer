extern crate core;

use axum::extract::State;
use http::StatusCode;
use tokio::sync::broadcast;

pub mod sql_products;
pub mod user;
pub mod product;
pub mod foto;
pub mod model;
pub mod zamowienia;
pub mod auth;
pub mod odleglosci_mapa;

#[derive(Clone)]
pub struct AppState {
    // pub tx: tokio::sync::mpsc::Sender<()>,
    pub db: sqlx::sqlite::SqlitePool,
    pub ws_broadcast_tx: broadcast::Sender<String>,
    // pub pepper_key: String,
}

pub async fn health_check_handler(
    State(state): State<AppState>,
) -> Result<(StatusCode, &'static str), StatusCode> {
    // Sprawdzenie połączenia z bazą SQLite
    let db_healthy = sqlx::query("SELECT 1")
        .execute(&state.db)
        .await
        .is_ok();

    if !db_healthy {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }

    Ok((StatusCode::OK, "HEALTHY"))
}