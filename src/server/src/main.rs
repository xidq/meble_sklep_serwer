pub mod requests;
pub mod response;
pub mod register;
pub mod websoc;
mod tests;
mod router;

use crate::router::routing::build_router;
use sqlite_serv::sql::AppState;
use sqlite_serv::{FILES_LOCATION, PEPPER_KEY};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use std::net::SocketAddr;
use std::str::FromStr;
use tokio::net::TcpListener;
use tokio::sync::broadcast;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    dotenvy::dotenv().ok();

    // =========================================================================
    // 🔥 SZYBKA DIAGNOSTYKA ŚRODOWISKA I ŚCIEŻEK W DOCKERZE
    // =========================================================================
    println!("\n=== [DIAGNOSTYKA STARTU APLIKACJI] ===");

    // 1. Sprawdzenie katalogu roboczego (CWD)
    match std::env::current_dir() {
        Ok(cwd) => println!("📍 Bieżący katalog roboczy (CWD): {:?}", cwd),
        Err(e) => println!("⚠️ Błąd odczytu CWD: {:?}", e),
    }

    // 2. Odczyt zmiennych środowiskowych
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "BRAK ZMIENNEJ".to_string());
    let files_url = std::env::var("FILES_URL").unwrap_or_else(|_| "BRAK ZMIENNEJ".to_string());
    println!("💾 DATABASE_URL wczytany jako: \"{}\"", db_url);
    println!("📁 FILES_URL wczytany jako: \"{}\"", files_url);

    // 3. Analiza ścieżki SQLite i test zapisu w folderze
    if db_url.starts_with("sqlite://") || db_url.starts_with("sqlite:") {
        let path_str = db_url
            .trim_start_matches("sqlite://")
            .trim_start_matches("sqlite:");

        if path_str != ":memory:" {
            let db_path = std::path::Path::new(path_str);

            // Wyznaczenie pełnej, absolutnej ścieżki
            let absolute_db_path = if db_path.is_absolute() {
                db_path.to_path_buf()
            } else if let Ok(cwd) = std::env::current_dir() {
                cwd.join(db_path)
            } else {
                db_path.to_path_buf()
            };

            println!("🔍 Pełna bezwzględna ścieżka do bazy: {:?}", absolute_db_path);

            if let Some(parent_dir) = absolute_db_path.parent() {
                println!("📂 Folder nadrzędny bazy danych: {:?}", parent_dir);
                let exists = parent_dir.exists();
                println!("❓ Czy ten folder istnieje? {}", if exists { "TAK" } else { "NIE (SQLite się wyłoży!)" });

                if exists {
                    // Próbujemy zapisać losowy plik, żeby sprawdzić uprawnienia systemu plików
                    let test_file_path = parent_dir.join(".write_test_docker");
                    match std::fs::write(&test_file_path, "test") {
                        Ok(_) => {
                            println!("✅ Uprawnienia zapisu w folderze: OK (zapis pomyślny)");
                            let _ = std::fs::remove_file(test_file_path);
                        }
                        Err(err) => {
                            println!("❌ BRAK UPRAWNIEŃ DO ZAPISU w folderze! Błąd: {:?}", err);
                        }
                    }
                }
            }
        }
    } else {
        println!("⚠️ DATABASE_URL nie wygląda na SQLite.");
    }
    println!("=================================================\n");
    // =========================================================================


    sqlite_serv::auth::jwt::initialize_jwt_secret();
    let pepper_key = std::env::var("PEPPER_KEY").expect("Brak PEPPER_KEY w .env");
    PEPPER_KEY.set(pepper_key).expect("Nie udało się zainicjalizować PEPPER_KEY");

    let files_location = std::env::var("FILES_URL").expect("Brak FILES_URL w .env");
    FILES_LOCATION.set(files_location).expect("Nie udało się zainicjalizować FILES_URL");

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL musi być ustawiona");

    // #[cfg(docker)]
    // if db_url.starts_with("sqlite://") {
    //     let path_str = db_url.trim_start_matches("sqlite://");
    //     if let Some(parent_dir) = Path::new(path_str).parent() {
    //         if !parent_dir.exists() && parent_dir.as_os_str() != "" {
    //             fs::create_dir_all(parent_dir)?;
    //             println!("Utworzono katalog dla bazy danych: {:?}", parent_dir);
    //         }
    //     }
    // }

    // 4. BEZPIECZNA KONFIGURACJA SQLITE DLA DOCKERA
    // Zamiast prostego .connect(), konfigurujemy bazę do stabilnej pracy w kontenerze
    #[cfg(docker)]
    let connection_options = SqliteConnectOptions::from_str(&db_url)?
        .create_if_missing(true) // Tworzy plik bazy automatycznie, jeśli go nie ma
        .journal_mode(SqliteJournalMode::Wal) // Tryb WAL - kluczowy przy wielu zapytaniach
        .busy_timeout(std::time::Duration::from_secs(5)); // Unikamy błędów "database is locked"
    // let pool_users = SqlitePoolOptions::new()
    //     .max_connections(5)
    //     .connect(&db_users_url)
    //     .await?;

    // nie szyf db
    // let pool = SqlitePoolOptions::new()
    //     .max_connections(5)
    //     .connect(&db_url)
    //     .await?;
    let pool = if cfg!(docker) {
        let connection_options = SqliteConnectOptions::from_str(&db_url)?
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .busy_timeout(std::time::Duration::from_secs(5));

        SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(connection_options) // <--- TUTAJ używamy opcji!
            .await?
    } else {
        SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await?
    };
    
    // sqlx::migrate!("../../migrations").run(&pool).await?;
    println!("Uruchamianie migracji dla bazy danych...");
    sqlx::migrate!("../../migrations/data").run(&pool).await?;

    println!("Wszystkie bazy danych zostały pomyślnie zsynchronizowane!");

    println!("Migracje zakończone sukcesem.");
    // let (tx, mut rx) = tokio::sync::mpsc::channel::<()>(32);

    // Tworzymy kanał broadcast (przesyłamy String, czyli zserializowany JSON z produktami)
    let (ws_broadcast_tx, _) = broadcast::channel::<String>(16);

    // Background worker - działa w nieskończonej pętli w tle
    // let (worker_produkty, worker_img) = (pool.clone(),pool.clone());
    // tokio::spawn(async move {
    //     while rx.recv().await.is_some() {
    //         // Przetwarzanie zdjęć
    //         if let Err(e) = image_database_compare_and_sht(&worker_produkty, &worker_img).await {
    //             eprintln!("Błąd w tle: {}", e);
    //         }
    //         println!("Zakończona pętla zdjęć");
    //     }
    // });

    // Zamykamy pulę w naszym stanie aplikacji

    let state = AppState { /* tx ,*/ db: pool , ws_broadcast_tx};


    let app = build_router(state);

    // #[cfg(docker)]
    // let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    // #[cfg(docker)]
    // let addr_str = format!("0.0.0.0:{}", port);
    // #[cfg(docker)]
    let addr = if cfg!(docker) {
        let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
        let addr_str = format!("0.0.0.0:{}", port);
        SocketAddr::from_str(&addr_str)
            .unwrap_or_else(|_| SocketAddr::from(([0, 0, 0, 0], 8080)))
    } else {
        SocketAddr::from(([127, 0, 0, 1], 8080))
    };
    // let addr = SocketAddr::from_str(&addr_str)
    //     .unwrap_or_else(|_| SocketAddr::from(([0, 0, 0, 0], 8080)));
    //
    // let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("Serwer działa na http://{}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    // axum::serve(listener, app).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>()
    )
        .await
        ?;
    Ok(())
}

