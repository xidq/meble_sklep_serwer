pub mod requests;
pub mod response;
pub mod register;
pub mod websoc;
mod tests;
mod router;

use crate::router::routing::build_router;
use axum_server::tls_rustls::RustlsConfig;
use env_thingy::{OnceLockExt, CURRENT_ADDRESS, CURRENT_PORT, DATABASE_URL, FILES_LOCATION, FRONT_SERV_ADDRESS, GOVERNOR_BURST_SIZE, GOVERNOR_RATE_LIMIT, JWT_SECRET, PEPPER_KEY, TLS_CERT_PATH, TLS_KEY_PATH};
use sqlite_serv::AppState;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use std::net::SocketAddr;
use std::str::FromStr;
use tokio::io::AsyncBufReadExt;
use tokio::sync::broadcast;
use colored;
use colored::Colorize;

/// Main fn of such server
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{

    load_env_data()?;

    let db_url = DATABASE_URL.v("");

    let pool = if cfg!(docker) {
        let connection_options = SqliteConnectOptions::from_str(db_url)?
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
            .connect(db_url)
            .await?
    };

    println!("Uruchamianie migracji dla bazy danych...");
    sqlx::migrate!("../../migrations/data").run(&pool).await?;

    println!("{}","Wszystkie bazy danych zostały pomyślnie zsynchronizowane!".green());

    println!("{}","Migracje zakończone sukcesem.".green());

    let (ws_broadcast_tx, _) = broadcast::channel::<String>(16);


    let state = AppState { /* tx ,*/ db: pool , ws_broadcast_tx};

    let app = build_router(state);
    // let app = Router::new().route("/", get(root));

    let rust_port = CURRENT_PORT.v("CURRENT_PORT not set");
    let rust_address = CURRENT_ADDRESS.v("CURRENT_ADDRESS not set");

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



    let config = RustlsConfig::from_pem_file(TLS_CERT_PATH.v(""),TLS_KEY_PATH.v("")).await?;

    println!("Serwer działa na https://{}", addr);


    // axum_server::bind_rustls(addr, config)
    //     // ta fn po app to bo https nie będzie współpracował z rate limiterem
    //     .serve(app.into_make_service_with_connect_info::<SocketAddr>())
    //     .await?;
    // Ok(())

    // ---------------------------------------------------------------
    println!("Write '{}' or press {}, to shut down such server.", "exit".bold().blue(), "Ctrl+C".bold().blue());

    // Zadanie nasłuchujące na konsolę (wpisanie 'exit' lub 'stop')
    let stdin_future = async {
        let stdin = tokio::io::stdin();
        let reader = tokio::io::BufReader::new(stdin);
        let mut lines = reader.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            let trimmed = line.trim();
            if trimmed.eq_ignore_ascii_case("exit") || trimmed.eq_ignore_ascii_case("stop") {
                println!("{}", "Otrzymano komendę z konsoli. Zamykanie serwera...".yellow());
                break;
            } else {
                println!("Nieznana komenda: '{}'. Wpisz 'exit', aby wyłączyć.", trimmed);
            }
        }
    };

    // Zadanie nasłuchujące na Ctrl+C
    let ctrl_c_future = async {
        tokio::signal::ctrl_c().await.expect("Nie udało się nasłuchiwać sygnału Ctrl+C");
        println!("\n{}", "Otrzymano sygnał Ctrl+C. Zamykanie serwera...".yellow());
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

    dotenvy::dotenv()?;
    
    println!("{}", "Loading Environment Data...".bold().blue());
    let pepper_key = std::env::var("PEPPER_KEY")?;
    PEPPER_KEY.set(pepper_key).map_err(|e| anyhow::anyhow!("Nie udało się zainicjalizować PEPPER_KEY: {}",e))?;
    println!("PEPPER_KEY: {}", "Ok!".bold().green());

    let files_location = std::env::var("FILES_URL")?;
    FILES_LOCATION.set(files_location).map_err(|e| anyhow::anyhow!("Nie udało się zainicjalizować FILES_LOCATION: {}", e))?;
    println!("FILES_URL: {}", "Ok".bold().green());

    let front_serv_port = std::env::var("CURRENT_RUST_SERVER_PORT")?;
    CURRENT_PORT.set(front_serv_port).map_err(|e| anyhow::anyhow!("Nie udało się zainicjalizować CURRENT_PORT: {}",e))?;
    println!("CURRENT_RUST_SERVER_PORT: {}", "Ok".bold().green());

    let front_serv_address = std::env::var("CURRENT_RUST_SERVER_ADRES")?;
    CURRENT_ADDRESS.set(front_serv_address).map_err(|e| anyhow::anyhow!("Nie udało się zainicjalizować CURRENT_ADDRESS: {}",e))?;
    println!("CURRENT_RUST_SERVER_ADRES: {}", "Ok".bold().green());

    let front_serv = std::env::var("FRONTEND_SERVER")?;
    FRONT_SERV_ADDRESS.set(front_serv).map_err(|e| anyhow::anyhow!("Nie udało się zainicjalizować FRONT_SERV_ADRESS: {}",e))?;
    println!("FRONTEND_SERVER: {}", "Ok".bold().green());

    let jwt = std::env::var("JWT_SECRET_KEY")?;
    JWT_SECRET.set(jwt.into_bytes()).map_err(|e| anyhow::anyhow!("Nie udało się zainicjalizować JWT_SECRET: {:?}",e))?;
    println!("JWT_SECRET_KEY: {}", "Ok".bold().green());

    let database = std::env::var("DATABASE_URL")?;
    DATABASE_URL.set(database).map_err(|e| anyhow::anyhow!("Nie udało się zainicjalizować DATABASE_URL: {}",e))?;
    println!("DATABASE_URL: {}", "Ok".bold().green());

    let cert = std::env::var("RUST_TLS_CERT")?;
    TLS_CERT_PATH.set(cert).map_err(|e| anyhow::anyhow!("Nie udało się zainicjalizować TLS_CERT_PATH: {}",e))?;
    println!("RUST_TLS_CERT: {}", "Ok".bold().green());

    let key = std::env::var("RUST_TLS_KEY")?;
    TLS_KEY_PATH.set(key).map_err(|e| anyhow::anyhow!("Nie udało się zainicjalizować TLS_KEY_PATH: {}",e))?;
    println!("RUST_TLS_KEY: {}", "Ok".bold().green());

    let burst = std::env::var("GOV_BURST_SIZE")?;
    let burst_parse: u32 = match burst.parse::<u32>(){
        Ok(x) => {x}
        Err(e) => {
            println!("GOV_BURST_SIZE: {}: {}. Using default: 5", "Error parsing burst limit".bold().red(), e);
            5_u32
        }
    };
    GOVERNOR_BURST_SIZE.set(burst_parse).map_err(|e| anyhow::anyhow!("Nie udało się zainicjalizować GOVERNOR_BURST_SIZE: {}",e))?;
    println!("GOV_BURST_SIZE: {}", "Ok".bold().green());


    // println!("loading env data GOV_RATE_LIMIT:");
    let rate_limit = std::env::var("GOV_RATE_LIMIT")?;
    let rate_limit_parse:u64 = match rate_limit.parse::<u64>(){
        Ok(x) => {x}
        Err(e) => {
            println!("GOV_RATE_LIMIT: {}: {}. Using default: 2", "Error parsing rate limit".bold().red(), e);
            2_u64
        }
    };
    GOVERNOR_RATE_LIMIT.set(rate_limit_parse)
        .map_err(|e| anyhow::anyhow!("Nie udało się zainicjalizować GOV_RATE_LIMIT: {}",e))?;
    println!("GOV_RATE_LIMIT: {}", "Ok".bold().green());

    println!("{}","-------------".blue().bold());

    Ok(())
}