use axum::Json;
use crate::cart::enumy::{CartItem, CartSummary};

pub async fn calculate_cart(Json(_items): Json<Vec<CartItem>>) -> Json<CartSummary> {
    Json(CartSummary { total_netto: 0.0, total_vat: 0.0, total_brutto: 0.0, total_volume_m3: 0.0, shipping_cost: 0.0, grand_total: 0.0 })
}