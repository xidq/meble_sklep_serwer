use crate::websoc::websocet;
use axum::extract::DefaultBodyLimit;
use axum::routing::{get, post, put};
use axum::Router;
use sqlite_serv::AppState;
use std::sync::Arc;
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::key_extractor::SmartIpKeyExtractor;
use tower_governor::GovernorLayer;
use tower_http::cors::{Any, CorsLayer};

/// Function that handles routing from external server
/// [GET, POST, PUT and DELETE]
pub fn build_router(state: AppState) -> Router {

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
        // .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        // .allow_headers([AUTHORIZATION, CONTENT_TYPE]);

    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .key_extractor(SmartIpKeyExtractor) // <-- tu
            .per_second(2)
            .burst_size(5)
            // .use_headers()
            .finish()
            .unwrap()
    );
    let governor_layer = GovernorLayer::new(governor_conf);
    
    Router::new()
        .route(
            "/",
            get(test)
        )
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
            "/wss",
            get(websocet::ws_handler)
        )
        .route(
            "/api/order",
            post(sqlite_serv::zamowienia::post::handle_put_order_new)
        )
        .route(
            "/api/images/upload/{item_name_id}",
            post(sqlite_serv::foto::upload::handler_image_upload_to_server)
        )
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024))
        // rate limiter jest na innym serwerze, a komunikacja pomiędzy serwerami fajnie jakby była nie ograniczona
        // rate-limiter
        .layer(governor_layer)
        .layer(cors)
        .with_state(state)

}

async fn test() -> &'static str {
    "no siema"
}