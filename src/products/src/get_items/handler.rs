use crate::get_items::enum_structs::{PRODUCTS_PATH};
use crate::get_items::lists::load_products;
use auth::claims_thingy::claims::claims_match;
use auth::file_handling::json_handling::file_serialisation;
use axum::response::IntoResponse;
use axum::Json;
use http::{HeaderMap, StatusCode};
use id_handling::create::get_new_id;
use id_handling::enums_structs::Product;

pub async fn add_product_handler(
    headers: HeaderMap,
    Json(new_product): Json<Product>,
) -> impl IntoResponse {

    let claims = match claims_match(&headers) {
        Ok(c) => c,
        Err(error_response) => return error_response, // Zwraca 401, 400 lub 403
    };

    if claims.role != "Admin" {
        return (StatusCode::FORBIDDEN, "Brak uprawnień administratora. Wymagana rola: Admin").into_response();
    }

    println!("Zgoda na edycję przyznana dla admina: {}", claims.username);

    let mut products = load_products();

    // match get_new_id(new_product,&mut products)? {
    //     Ok(_) => {}
    //     Err(error_response) => return error_response,
    // }
    match get_new_id(new_product, &mut products) {
        Ok(_) => {  }
        Err(error_response) => {
            return error_response.into_response();
        }
    }

    file_serialisation(PRODUCTS_PATH, products)
}
