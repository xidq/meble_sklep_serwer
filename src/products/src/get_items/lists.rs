use crate::get_items::enum_structs::{PRODUCTS_PATH};
use axum::Json;
use http::status::StatusCode;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use id_handling::enums_structs::Product;

pub async fn list_products() -> (StatusCode, Json<Vec<Product>>) {
    let products = load_products();
    (StatusCode::OK, Json(products))
}

pub fn load_products() -> Vec<Product> {
    let path = Path::new(PRODUCTS_PATH);
    if let Ok(file) = File::open(path) {
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).unwrap_or_else(|_| vec![])
    } else {
        vec![]
    }
}