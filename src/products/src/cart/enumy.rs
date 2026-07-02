use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CartItem {
    pub product_id: u64,
    pub quantity: u32,
}

#[derive(Serialize)]
pub struct CartSummary {
    pub total_netto: f64,
    pub total_vat: f64,
    pub total_brutto: f64,
    pub total_volume_m3: f64,
    pub shipping_cost: f64,
    pub grand_total: f64,
}