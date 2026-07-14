pub mod requests;
pub mod response;
pub mod register;
pub mod websoc;

use axum::extract::DefaultBodyLimit;
use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
use axum::http::Method;
use axum::routing::put;
use axum::{
    routing::{get, post}
    , Router,
};
use sqlite_serv::sql::AppState;
use sqlite_serv::PEPPER_KEY;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::cors::{Any, CorsLayer};
use websoc::websocet;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    dotenvy::dotenv().ok();
    sqlite_serv::auth::jwt::initialize_jwt_secret();
    let pepper_key = std::env::var("PEPPER_KEY").expect("Brak PEPPER_KEY w .env");
    PEPPER_KEY.set(pepper_key).expect("Nie udało się zainicjalizować PEPPER_KEY");

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL musi być ustawiona");

    #[cfg(docker)]
    if db_url.starts_with("sqlite://") {
        let path_str = db_url.trim_start_matches("sqlite://");
        if let Some(parent_dir) = Path::new(path_str).parent() {
            if !parent_dir.exists() && parent_dir.as_os_str() != "" {
                fs::create_dir_all(parent_dir)?;
                println!("Utworzono katalog dla bazy danych: {:?}", parent_dir);
            }
        }
    }

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

    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(2)
            .burst_size(5)
            .finish()
            .unwrap()
    );
    let governor_layer = GovernorLayer::new(governor_conf);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        // .allow_methods(Any)
        // .allow_headers(Any);
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE]); // TO JEST KLUCZOWE

    // router
    let app = Router::new()
        .route(
            "/usr/login",
            post(websocet::login_handler)
        )
        .route(
            "/usr/usr",
            post(sqlite_serv::user::post::handler_user_new)
                .get(sqlite_serv::user::get::handler_get_user_own_data)
                .delete(sqlite_serv::user::delete::handler_delete_user_by_user)
                .put(sqlite_serv::user::put::handle_edit_user_by_user)
        )
        .route(
            "/api/user/orders",
            get(sqlite_serv::zamowienia::get::handler_get_user_orders)
        )
        .route(
            "/admin/usr",
            post(sqlite_serv::user::post::handler_user_new)
                .put(sqlite_serv::user::put::handle_edit_user)
                .get(sqlite_serv::user::get::handler_user_get_list)
        )
        .route(
            "/admin/usr/{id}",
            get(sqlite_serv::user::get::handler_get_user_data_by_id) //get user data
                .delete(sqlite_serv::user::delete::handler_delete_user_by_id)
                // .put(sqlite_serv::user::put::handle_edit_user) //nie trza id, jest caly user
                // .delete(sqlite_serv::user::delete::handler_delete_user_by_id)
        )
        .route(
            "/api/products",
            get(sqlite_serv::product::get::handler_get_products_list)
                .post(sqlite_serv::product::post::handler_put_product_new) //nowy
        )
        .route(
            "/api/products/name_id/{name_id}",
            get(sqlite_serv::product::get::handler_get_products_data_by_nameid)
        )
        .route(
            "/api/products/{id}",
            put(sqlite_serv::product::put::handle_edit_product) //update
                .get(sqlite_serv::product::get::handler_get_products_data_by_id)
                .delete(sqlite_serv::product::delete::handler_delete_product_by_id)
        )
        .route(
            "/api/models",
            get(sqlite_serv::model::get::handler_get_models_list)
        )
        .route(
            "/api/models/{id}",
            get(sqlite_serv::model::get::handler_get_models_data_by_id)
        )
        .route(
            "/api/models/upload",
            get(sqlite_serv::model::upload::handler_model_upload_to_server)
        )
        .route(
            "/ws",
            get(websocet::ws_handler)
        )
        .route(
            "/api/order",
            post(sqlite_serv::zamowienia::post::handle_put_order_new)
        )
        .route(
            "/api/images/upload/{item_name_id}",
            post(sqlite_serv::foto::upload::handler_image_upload_to_server))
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024))
        // rate-limiter
        .layer(governor_layer)
        .layer(cors)
        .with_state(state);

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

