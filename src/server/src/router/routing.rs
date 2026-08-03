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
        )
        .route(
            "/usr/self/orders",
            get(sqlite_serv::zamowienia::get::handler_get_user_orders)
        )
        .route(
            "/usr/self/password",
            put(sqlite_serv::user::put::handler_edit_user_password)
        )
        .route(
            "/usr/self/data",
            get(sqlite_serv::user::get::handler_get_user_profile) // todo!("ogarnąć rozpiździel w handlerach")
                .put(sqlite_serv::user::put::handler_edit_user_profile)
                .delete(sqlite_serv::user::delete::handler_delete_user_profile)
        )
        .route(
            "/admin/usr",
            post(sqlite_serv::user::post::handler_user_new)
                .put(sqlite_serv::user::put::handle_admin_edit_user_data)
                .get(sqlite_serv::user::get::handler_user_get_list)
        )
        .route(
            "/admin/orders",
                get(sqlite_serv::zamowienia::get::handler_admin_get_order_lists)
        )
        .route(
            "/admin/orders/{order_id}",
                get(sqlite_serv::zamowienia::get::handler_admin_get_order_item_by_id)
                    .put(sqlite_serv::zamowienia::put::handle_admin_edit_orders)
        )
        .route(
            "/admin/usr/{name_id}",
            get(sqlite_serv::user::get::handler_get_user_data_by_nameid) //get user data
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
            "/api/models/list",
            get(sqlite_serv::model::get::handler_get_models_list)
        )
        .route(
            "/api/models/data/{id}",
            get(sqlite_serv::model::get::handler_get_models_data_by_id)
        )
        .route(
            "/api/models/upload/{id}",
            post(sqlite_serv::model::upload::handler_model_upload_to_server)
        )
        .route(
            "/api/models/refresh/{id}",
            post(sqlite_serv::model::post::handler_refresh_model_json_at_front)
        )
        .route(
            "/api/models/refresh",
            post(sqlite_serv::model::post::handler_refresh_all_models_json_at_front)
        )
        .route(
            "/api/admin/sync/models",
            get(sqlite_serv::model::get::handler_sync_models_json)
        )
        .route(
            "/wss",
            get(websocet::wss_handler)
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