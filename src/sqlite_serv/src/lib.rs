extern crate core;

use std::sync::OnceLock;
use tokio::sync::broadcast;

pub mod sql_products;
pub mod user;
pub mod product;
pub mod foto;
pub mod model;
pub mod zamowienia;
pub mod auth;


pub static PEPPER_KEY: OnceLock<String> = OnceLock::new();

/// Path for folder with external files eg images.
pub static FILES_LOCATION: OnceLock<String> = OnceLock::new();

#[derive(Clone)]
pub struct AppState {
    // pub tx: tokio::sync::mpsc::Sender<()>,
    pub db: sqlx::sqlite::SqlitePool,
    pub ws_broadcast_tx: broadcast::Sender<String>,
    // pub pepper_key: String,
}