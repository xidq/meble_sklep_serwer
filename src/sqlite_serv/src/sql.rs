use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppState {
    // pub tx: tokio::sync::mpsc::Sender<()>,
    pub db: sqlx::sqlite::SqlitePool,
    pub ws_broadcast_tx: broadcast::Sender<String>,
    // pub pepper_key: String,
}