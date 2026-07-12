pub mod post;
pub mod put;
pub mod delete;
pub mod get;
pub mod upload;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use sqlx::{Column, FromRow, Row};

type Lod = String; // np. "LOD0", "LOD1"
#[derive(Serialize, Deserialize, /*FromRow,*/ Debug, Clone)]
pub struct Model{
    pub product_id: i64,
    pub texture_ao: Option<String>,
    #[serde(flatten)]
    pub model: BTreeMap<Lod, String>,
}

// impl<'r> FromRow<'r, sqlx::sqlite::SqliteRow> for Model {
//     fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
//         let product_id: i64 = row.try_get("product_id")?;
//         let textura_ao: Option<String> = row.try_get("textura_ao")?;
//
//         let mut modele = BTreeMap::new();
//
//         // Dynamicznie sprawdzamy wszystkie kolumny wiersza w poszukiwaniu LOD-ów
//         // (np. "LOD0", "LOD1", "LOD2" itd.)
//         for column in row.columns() {
//             let col_name = column.name();
//             if col_name.starts_with("LOD") || col_name.starts_with("lod") {
//                 // Jeśli kolumna istnieje i ma wartość (nie jest nullem), dodajemy do mapy
//                 if let Ok(Some(val)) = row.try_get::<Option<String>, _>(col_name) {
//                     modele.insert(col_name.to_string(), val);
//                 }
//             }
//         }
//
//         Ok(Model {
//             product_id,
//             texture_ao: textura_ao,
//             model: modele,
//         })
//     }
// }
impl<'r> FromRow<'r, sqlx::sqlite::SqliteRow> for Model {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        let product_id: i64 = row.try_get("product_id")?;
        let texture_ao: Option<String> = row.try_get("texture_ao")?;

        // 1. Wyciągamy kolumnę "model" jako zwykły String
        let model_str: String = row.try_get("model")?;

        // 2. Przerabiamy ten JSON z powrotem na mapę LOD-ów
        let model: BTreeMap<Lod, String> = serde_json::from_str(&model_str)
            .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

        Ok(Model {
            product_id,
            texture_ao,
            model,
        })
    }
}