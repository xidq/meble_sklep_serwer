use crate::sql::{delete_product, get_product_with_images, insert_product, list_product_ids_and_names, list_products_with_images, update_product, AppState, Product, ProductWithImages, Rozdzielczosci};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use axum_macros::debug_handler;

// ============================================================
// 1. LISTA WSZYSTKICH PRODUKTÓW Z OBRAZKAMI
// ============================================================
#[debug_handler]
pub async fn handle_list_all_products(
    State(state): State<AppState>,
) -> Result<Json<Vec<ProductWithImages>>, (StatusCode, String)> {
    list_products_with_images(&state.db)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

// ============================================================
// 2. LISTA TYLKO ID I NAZW
// ============================================================
pub async fn handle_list_ids_and_names(
    State(state): State<AppState>,
) -> Result<Json<Vec<(u64, String)>>, (StatusCode, String)> {
    list_product_ids_and_names(&state.db)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

// ============================================================
// 3. POBIERZ PRODUKT PO ID (Z OBRAZKAMI)
// ============================================================
#[debug_handler]
pub async fn handle_get_product(
    State(state): State<AppState>,
    Path(product_id): Path<i64>,
) -> Result<Json<ProductWithImages>, (StatusCode, String)> {
    match get_product_with_images(product_id, &state.db).await {
        Ok(data) => Ok(Json(data)),
        Err(sqlx::Error::RowNotFound) => {
            Err((StatusCode::NOT_FOUND, "Product not found".to_string()))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

// ============================================================
// 4. DODAJ NOWY PRODUKT (BEZ OBRAZKÓW)
// ============================================================
// #[derive(serde::Deserialize)]
// pub struct CreateProductRequest {
//     pub name: String,
//     pub price: f32,
//     pub vat: f32,
//     pub description_pl: String,
//     pub description_en: String,
//     pub model_3d: Option<String>,
//     pub texture_ao: Option<String>,
//     pub texture_normal: Option<String>,
//     pub width: f32,
//     pub height: f32,
//     pub depth: f32,
// }
#[derive(serde::Deserialize)]
pub struct CreateProductRequest {
    #[serde(flatten)]
    pub product: Product,
    pub images: Vec<(Rozdzielczosci, String)>,
}
pub async fn handle_create_product(
    State(state): State<AppState>,
    Json(payload): Json<CreateProductRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, String)> {
    //
    // let dane = Product{
    //     &payload.name,
    //     payload.price,
    //     payload.vat,
    //     &payload.description_pl,
    //     &payload.description_en,
    //     payload.model_3d.as_deref(),
    //     payload.texture_ao.as_deref(),
    //     payload.texture_normal.as_deref(),
    //     payload.width,
    //     payload.height,
    //     payload.depth,
    // };
    let id = insert_product(
        &state.db,
        payload.product,
        payload.images,
    )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "id": id })),
    ))
}

// ============================================================
// 5. AKTUALIZUJ CAŁY PRODUKT (PUT)
// ============================================================
#[derive(serde::Deserialize)]
pub struct UpdateProductRequest {
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

pub async fn handle_update_product(
    State(state): State<AppState>,
    Path(product_id): Path<i64>,
    Json(payload): Json<Product>,
) -> Result<StatusCode, (StatusCode, String)> {
    update_product(
        &state.db,
        product_id,
        &payload,
    )
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => (StatusCode::NOT_FOUND, "Product not found".to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================
// 6. CZĘŚCIOWA AKTUALIZACJA (PATCH)
// ============================================================
#[derive(serde::Deserialize, Default)]
pub struct PatchProductRequest {
    pub name: Option<String>,
    pub price: Option<f32>,
    pub vat: Option<f32>,
    pub description_pl: Option<String>,
    pub description_en: Option<String>,
    pub model_3d: Option<Option<String>>,
    pub texture_ao: Option<Option<String>>,
    pub texture_normal: Option<Option<String>>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub depth: Option<f32>,
}

// pub async fn handle_patch_product(
//     State(state): State<AppState>,
//     Path(product_id): Path<u64>,
//     Json(payload): Json<PatchProductRequest>,
// ) -> Result<StatusCode, (StatusCode, String)> {
//     patch_product(
//         &state.db,
//         product_id,
//         payload.name.as_deref(),
//         payload.price,
//         payload.vat,
//         payload.description_pl.as_deref(),
//         payload.description_en.as_deref(),
//         payload.model_3d.map(|v| v.as_deref()),
//         payload.texture_ao.map(|v| v.as_deref()),
//         payload.texture_normal.map(|v| v.as_deref()),
//         payload.width,
//         payload.height,
//         payload.depth,
//     )
//         .await
//         .map_err(|e| match e {
//             sqlx::Error::RowNotFound => (StatusCode::NOT_FOUND, "Product not found".to_string()),
//             _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
//         })?;
//
//     Ok(StatusCode::NO_CONTENT)
// }

// ============================================================
// 7. USUŃ PRODUKT
// ============================================================
pub async fn handle_delete_product(
    State(state): State<AppState>,
    Path(product_id): Path<i64>,
) -> Result<StatusCode, (StatusCode, String)> {
    delete_product(&state.db, product_id)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => (StatusCode::NOT_FOUND, "Product not found".to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================
// 8. DODAJ OBRAZEK DO PRODUKTU
// ============================================================
#[derive(serde::Deserialize)]
pub struct AddImageRequest {
    pub resolution: Rozdzielczosci,
    pub path: String,
}

// pub async fn handle_add_image(
//     State(state): State<AppState>,
//     Path(product_id): Path<u64>,
//     Json(payload): Json<AddImageRequest>,
// ) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, String)> {
//     let image_id = add_product_image(
//         &state.db,
//         product_id,
//         payload.resolution,
//         &payload.path,
//     )
//         .await
//         .map_err(|e| match e {
//             sqlx::Error::RowNotFound => (StatusCode::NOT_FOUND, "Product not found".to_string()),
//             _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
//         })?;
//
//     Ok((
//         StatusCode::CREATED,
//         Json(serde_json::json!({ "image_id": image_id })),
//     ))
// }

// async fn add_product_image(p0: &SqlitePool, p1: u64, p2: Rozdzielczosci, p3: &String) {
//     todo!()
// }

// ============================================================
// 9. USUŃ OBRAZEK
// ============================================================
// pub async fn handle_delete_image(
//     State(state): State<AppState>,
//     Path(image_id): Path<i64>,
//     rozdzielczosc: Rozdzielczosci,
// ) -> Result<StatusCode, (StatusCode, String)> {
//     delete_product_image(&state.db, image_id, rozdzielczosc)
//         .await
//         .map_err(|e| match e {
//             sqlx::Error::RowNotFound => (StatusCode::NOT_FOUND, "Image not found".to_string()),
//             _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
//         })?;
//
//     Ok(StatusCode::NO_CONTENT)
// }