extern crate core;

use tokio::sync::broadcast;

pub mod sql_products;
pub mod user;
pub mod product;
pub mod foto;
pub mod model;
pub mod zamowienia;
pub mod auth;
mod odleglosci_mapa;

#[derive(Clone)]
pub struct AppState {
    // pub tx: tokio::sync::mpsc::Sender<()>,
    pub db: sqlx::sqlite::SqlitePool,
    pub ws_broadcast_tx: broadcast::Sender<String>,
    // pub pepper_key: String,
}