mod image_ops;
pub mod get;
mod put;
mod post;
mod delete;
pub mod upload;

use crate::FILES_LOCATION;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};
use std::collections::BTreeMap;

type Rozdzielczosc = String; // np. "16", "32"
type WariantFoto = String;      // np. "var_1", "var_2"
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FotoData {
    product_id: i64,
    #[serde(flatten)]
    pub warianty_zdjec: BTreeMap<WariantFoto, BTreeMap<Rozdzielczosc, String>>,
}
impl<'r> FromRow<'r, sqlx::sqlite::SqliteRow> for FotoData {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        let product_id: i64 = row.try_get("product_id")?;

        // 1. Wyciągamy kolumnę "model" jako zwykły String
        let model_str: String = row.try_get("warianty_zdjec")?;

        // 2. Przerabiamy ten JSON z powrotem na mapę LOD-ów
        let warianty_zdjec: BTreeMap< WariantFoto, BTreeMap<Rozdzielczosc, String>> = serde_json::from_str(&model_str)
            .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

        Ok(FotoData {
            product_id,
            warianty_zdjec,
        })
    }
}
pub fn get_items_prefix<'a>() -> &'a str {
    // println!("items prefix!!!");
    FILES_LOCATION.get().expect("FILES_LOCATION nie jest zainicjalizowany")
}