use serde::{Deserialize, Serialize};
use sqlx::FromRow;

pub trait ElementyJson{
    fn get_id(&self) -> i64;
    fn get_value(&self) -> u8;
    fn set_id(&mut self, id: i64);
}
// #[derive(Serialize, Deserialize, Clone, sqlx::FromRow)]
// pub struct Product {
//     pub(crate) id: u64,
//     pub(crate) name: String,
//     pub description: String,
//     pub price_netto: f64,
//     pub vat: f64, // np. 0.23 dla 23%
//     // Dane 3D i wymiary fizyczne
//     pub model_url: String,
//     pub width_cm: f64,
//     pub height_cm: f64,
//     pub depth_cm: f64,
//     pub suggested_render_scale: f32,
// }
#[derive(Debug, FromRow, Deserialize, Serialize, Clone)]
pub struct Product {
    pub id: i64,
    pub name_id: String,
    pub name: String,
    pub price: f32,
    pub vat: f32,
    pub description_pl: String,
    pub description_en: String,
    pub model_3d: Option<String>,
    pub texture_ao: Option<String>,
    pub texture_normal: Option<String>,
    pub width: f32,
    pub height: f32,
    pub depth: f32,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelData{
    pub name: String,
    pub path: String,
    pub uid: i64,
    pub texture_ao: String,
    pub texture_normal: String,
}
impl ElementyJson for Product {
    fn get_id(&self) -> i64 {self.id}
    fn get_value(&self) -> u8 { 101 }
    fn set_id(&mut self, id: i64) {self.id = id}
}
impl ElementyJson for ModelData {
    fn get_id(&self) -> i64 {self.uid}
    fn get_value(&self) -> u8 { 102 }
    fn set_id(&mut self, id: i64) {self.uid = id}

}