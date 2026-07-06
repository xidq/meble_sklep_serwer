pub mod requests;
pub mod response;
pub mod register;
pub mod websoc;

use axum::extract::DefaultBodyLimit;
use axum::routing::{delete, put};
use axum::{
    routing::{get, post}
    , Router,
};
use products::get_items::lists;
use register::handler;
use sqlite_serv::handler::{handle_create_product, handle_delete_product, handle_get_product, handle_list_all_products, handle_list_ids_and_names, handle_update_product};
use sqlite_serv::image_handling::{image_database_compare_and_sht, image_upload_to_server_handle};
use sqlite_serv::sql::AppState;
use sqlx::sqlite::SqlitePoolOptions;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::cors::{Any, CorsLayer};
use websoc::websocet;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    dotenvy::dotenv().ok();
    auth::jwt::initialize_jwt_secret();

    // let governor_conf = Arc::new(
    //     GovernorConfigBuilder::default()
    //         .per_second(2)
    //         .burst_size(5)
    //         .finish()
    //         .unwrap()
    // );
    let database_url = std::env::var("DATABASE_URL")
        .expect("Zmienna DATABASE_URL musi być ustawiona w pliku .env");

    // 2. Tworzymy pulę połączeń z bazą danych
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    sqlx::migrate!("../../migrations").run(&pool).await?;

    println!("Migracje zakończone sukcesem.");
    let (tx, mut rx) = tokio::sync::mpsc::channel::<()>(32);

    // Background worker - działa w nieskończonej pętli w tle
    let worker_pool = pool.clone();
    tokio::spawn(async move {
        while rx.recv().await.is_some() {
            // Przetwarzanie zdjęć
            if let Err(e) = image_database_compare_and_sht(&worker_pool).await {
                eprintln!("Błąd w tle: {}", e);
            }
        }
    });

    // Zamykamy pulę w naszym stanie aplikacji

    let state = AppState { tx, db: pool };

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
        .allow_methods(Any)
        .allow_headers(Any);

    // router
    let app = Router::new()
        .route("/api/login", post(websocet::login_handler))
        .route("/api/register", post(handler::register_handler))
        .route("/api/products", get(lists::list_products))
        .route("/api/products", post(products::get_items::handler::add_product_handler))
        .route("/api/models", get(models::handle::list_models))
        .route("/api/models", post(models::handle::model_data_storage))
        .route("/api/cart/calculate", post(products::cart::calc::calculate_cart))
        .route("/ws", get(websocet::ws_handler))
        .route("/products", get(handle_list_all_products))
        .route("/products/ids", get(handle_list_ids_and_names))
        .route("/products/{id}", get(handle_get_product))
        .route("/products", post(handle_create_product))
        .route("/products/{id}", put(handle_update_product))
        // .route("/products/:id", patch(handle_patch_product))
        .route("/products/{id}", delete(handle_delete_product))
        // .route("/products/:id/images", post(handle_add_image))
        // .route("/images/:id", delete(handle_delete_image))
        .route("/api/images/upload", post(image_upload_to_server_handle))
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024))
        // rate-limiter
        .layer(governor_layer)
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
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