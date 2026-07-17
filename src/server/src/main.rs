pub mod requests;
pub mod response;
pub mod register;
pub mod websoc;
mod tests;
mod router;

use crate::router::routing::build_router;
use axum_server::tls_rustls::RustlsConfig;
use sqlite_serv::{AppState, FRONT_SERV_ADRESS};
use sqlite_serv::{FILES_LOCATION, PEPPER_KEY};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use std::net::SocketAddr;
use std::str::FromStr;
use tokio::sync::broadcast;

/// Main fn of such server
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{

    dotenvy::dotenv().ok();

    sqlite_serv::auth::jwt::initialize_jwt_secret();
    let pepper_key = std::env::var("PEPPER_KEY").expect("Brak PEPPER_KEY w .env");
    PEPPER_KEY.set(pepper_key).expect("Nie udało się zainicjalizować PEPPER_KEY");

    let files_location = std::env::var("FILES_URL").expect("Brak FILES_URL w .env");
    FILES_LOCATION.set(files_location).expect("Nie udało się zainicjalizować FILES_URL");

    let front_serv = std::env::var("FRONTEND_SERVER").expect("Brak FRONTEND_SERVER w .env");
    FRONT_SERV_ADRESS.set(front_serv).expect("Nie udało się zainicjalizować FRONTEND_SERVER");

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL musi być ustawiona");

    let pool = if cfg!(docker) {
        let connection_options = SqliteConnectOptions::from_str(&db_url)?
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .busy_timeout(std::time::Duration::from_secs(5));

        SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(connection_options)
            .await?
    } else {
        SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await?
    };

    println!("Uruchamianie migracji dla bazy danych...");
    sqlx::migrate!("../../migrations/data").run(&pool).await?;

    println!("Wszystkie bazy danych zostały pomyślnie zsynchronizowane!");

    println!("Migracje zakończone sukcesem.");

    let (ws_broadcast_tx, _) = broadcast::channel::<String>(16);


    let state = AppState { /* tx ,*/ db: pool , ws_broadcast_tx};

    let app = build_router(state);
    // let app = Router::new().route("/", get(root));

    let port = std::env::var("CURRENT_RUST_SERVER_PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = if cfg!(docker) {
        let addr_str = format!("0.0.0.0:{}", port);
        SocketAddr::from_str(&addr_str)
            .unwrap_or_else(|_| SocketAddr::from(([0, 0, 0, 0], 8080)))
    } else {
        SocketAddr::from(([192,168,0,111], 8080))
    };
    // let addr = SocketAddr::from(([0, 0, 0, 0], 8443));

    // let cert_path = std::env::var("RUST_TLS_CERT").unwrap_or_else(|_| "./server.crt".to_string());
    // let key_path = std::env::var("RUST_TLS_KEY").unwrap_or_else(|_| "./server.key".to_string());



    let config = RustlsConfig::from_pem_file("cert.pem","key.pem").await?;

    println!("Serwer działa na https://{}", addr);


    axum_server::bind_rustls(addr, config)
        // ta fn po app to bo https nie będzie współpracował z rate limiterem
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await?;
    Ok(())
}
