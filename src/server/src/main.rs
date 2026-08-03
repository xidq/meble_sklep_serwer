pub mod requests;
pub mod response;
pub mod register;
pub mod websoc;
mod tests;
mod router;

use crate::router::routing::build_router;
use axum_server::tls_rustls::RustlsConfig;
use sqlite_serv::AppState;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use std::net::SocketAddr;
use std::str::FromStr;
use tokio::io::AsyncBufReadExt;
use tokio::sync::broadcast;
use env_thingy::{CURRENT_ADDRESS, CURRENT_PORT, DATABASE_URL, FILES_LOCATION, FRONT_SERV_ADRESS, JWT_SECRET, PEPPER_KEY, TLS_CERT_PATH, TLS_KEY_PATH};

/// Main fn of such server
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{

    dotenvy::dotenv().ok();

    load_env_data()?;
    // let pepper_key = std::env::var("PEPPER_KEY").expect("Brak PEPPER_KEY w .env");
    // PEPPER_KEY.set(pepper_key).expect("Nie udało się zainicjalizować PEPPER_KEY");
    //
    // let files_location = std::env::var("FILES_URL").expect("Brak FILES_URL w .env");
    // FILES_LOCATION.set(files_location).expect("Nie udało się zainicjalizować FILES_URL");
    //
    // let front_serv = std::env::var("FRONTEND_SERVER").expect("Brak FRONTEND_SERVER w .env");
    // FRONT_SERV_ADRESS.set(front_serv).expect("Nie udało się zainicjalizować FRONTEND_SERVER");

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

    let rust_port = CURRENT_PORT.get().expect("CURRENT_PORT not set");
    let rust_address = CURRENT_ADDRESS.get().expect("CURRENT_ADDRESS not set");

    let addr = if cfg!(docker) {
        let addr_str = format!("0.0.0.0:{}", rust_port);
        SocketAddr::from_str(&addr_str)
            .unwrap_or_else(|_| SocketAddr::from(([0, 0, 0, 0], 8080)))
    } else {
        // SocketAddr::from(([192,168,0,111], 8080))

        let addr_str = format!("{}:{}", rust_address, rust_port);
        SocketAddr::from_str(&addr_str)
            .unwrap_or_else(|_| SocketAddr::from(([0, 0, 0, 0], 8080)))
    };
    // let addr = SocketAddr::from(([0, 0, 0, 0], 8443));

    // let cert_path = std::env::var("RUST_TLS_CERT").unwrap_or_else(|_| "./server.crt".to_string());
    // let key_path = std::env::var("RUST_TLS_KEY").unwrap_or_else(|_| "./server.key".to_string());



    let config = RustlsConfig::from_pem_file(TLS_CERT_PATH.get().unwrap(),TLS_KEY_PATH.get().unwrap()).await?;

    println!("Serwer działa na https://{}", addr);


    // axum_server::bind_rustls(addr, config)
    //     // ta fn po app to bo https nie będzie współpracował z rate limiterem
    //     .serve(app.into_make_service_with_connect_info::<SocketAddr>())
    //     .await?;
    // Ok(())

    // ---------------------------------------------------------------
    println!("Wpisz 'exit' lub naciśnij Ctrl+C, aby wyłączyć serwer.");

    // Zadanie nasłuchujące na konsolę (wpisanie 'exit' lub 'stop')
    let stdin_future = async {
        let stdin = tokio::io::stdin();
        let reader = tokio::io::BufReader::new(stdin);
        let mut lines = reader.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            let trimmed = line.trim();
            if trimmed.eq_ignore_ascii_case("exit") || trimmed.eq_ignore_ascii_case("stop") {
                println!("Otrzymano komendę z konsoli. Zamykanie serwera...");
                break;
            } else {
                println!("Nieznana komenda: '{}'. Wpisz 'exit', aby wyłączyć.", trimmed);
            }
        }
    };

    // Zadanie nasłuchujące na Ctrl+C
    let ctrl_c_future = async {
        tokio::signal::ctrl_c().await.expect("Nie udało się nasłuchiwać sygnału Ctrl+C");
        println!("\nOtrzymano sygnał Ctrl+C. Zamykanie serwera...");
    };

    // Odpalamy serwer axum w tle, a główne wątki/future pilnują sygnału zamknięcia
    let server_future = axum_server::bind_rustls(addr, config)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>());

    tokio::select! {
        _ = server_future => {}
        _ = stdin_future => {}
        _ = ctrl_c_future => {}
    }

    println!("Serwer został całkowicie wyłączony.");
    Ok(())
}
/// Initialize secrets
fn load_env_data() -> anyhow::Result<()>{

    println!("loading env data PEPPER_KEY. Status:");
    let pepper_key = std::env::var("PEPPER_KEY").expect("Brak PEPPER_KEY w .env");
    PEPPER_KEY.set(pepper_key).expect("Nie udało się zainicjalizować PEPPER_KEY");
    println!("Ok!");

    println!("loading env data FILES_URL. Status:");
    let files_location = std::env::var("FILES_URL").expect("Brak FILES_URL w .env");
    FILES_LOCATION.set(files_location).expect("Nie udało się zainicjalizować FILES_LOCATION");
    println!("Ok");

    println!("loading env data CURRENT_RUST_SERVER_PORT. Status:");
    let front_serv_port = std::env::var("CURRENT_RUST_SERVER_PORT").expect("Brak CURRENT_RUST_SERVER_PORT w .env");
    CURRENT_PORT.set(front_serv_port).expect("Nie udało się zainicjalizować CURRENT_PORT");
    println!("Ok");

    println!("loading env data CURRENT_RUST_SERVER_ADRES. Status:");
    let front_serv_address = std::env::var("CURRENT_RUST_SERVER_ADRES").expect("Brak CURRENT_RUST_SERVER_ADRES w .env");
    CURRENT_ADDRESS.set(front_serv_address).expect("Nie udało się zainicjalizować CURRENT_ADDRESS");
    println!("Ok");


    println!("loading env data FRONTEND_SERVER. Status:");
    let front_serv = std::env::var("FRONTEND_SERVER").expect("Brak FRONTEND_SERVER w .env");
    FRONT_SERV_ADRESS.set(front_serv).expect("Nie udało się zainicjalizować FRONT_SERV_ADRESS");
    println!("Ok");

    println!("loading env data JWT_SECRET_KEY. Status:");
    let jwt = std::env::var("JWT_SECRET_KEY").expect("Brak JWT_SECRET_KEY w .env");
    JWT_SECRET.set(jwt.into_bytes()).expect("Nie udało się zainicjalizować JWT_SECRET");
    println!("Ok");

    println!("loading env data DATABASE_URL. Status:");
    let database = std::env::var("DATABASE_URL").expect("Brak DATABASE_URL w .env");
    DATABASE_URL.set(database).expect("Nie udało się zainicjalizować DATABASE_URL");
    println!("Ok");

    println!("loading env data RUST_TLS_CERT. Status:");
    let cert = std::env::var("RUST_TLS_CERT").expect("Brak TLS_CERT_PATH w .env");
    TLS_CERT_PATH.set(cert).expect("Nie udało się zainicjalizować TLS_CERT_PATH");
    println!("Ok");

    println!("loading env data RUST_TLS_KEY. Status:");
    let key = std::env::var("RUST_TLS_KEY").expect("Brak TLS_KEY_PATH w .env");
    TLS_KEY_PATH.set(key).expect("Nie udało się zainicjalizować TLS_KEY_PATH");
    println!("Ok");

    Ok(())
}