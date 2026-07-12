use axum::extract::{Path, State};
use axum::Json;
use http::StatusCode;
use sqlx::SqlitePool;
use crate::product::Product;
use crate::sql::AppState;

pub async fn handle_edit_product(
    State(state): State<AppState>,
    Json(payload): Json<Product>,
) -> Result<StatusCode, (StatusCode, String)> {
    println!("Odebrano żądanie zmiany produktu id: {}", &payload.id);
    edit_product(
        &state.db,
        &payload,
    )
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => (StatusCode::NOT_FOUND, "Product not found".to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        })?;

    Ok(StatusCode::NO_CONTENT)
}
pub async fn edit_product(pool: &SqlitePool, product: &Product) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE products
        SET
            name_id = ?,
            name_pl = ?,
            name_en = ?,
            desc_pl = ?,
            desc_en = ?,
            wood_qua = ?,
            metal_qua = ?,
            glass_qua = ?,
            price = ?,
            width = ?,
            height = ?,
            depth = ?
        WHERE id = ?
        "#
    )
        .bind(&product.name_id)
        .bind(&product.name_pl)
        .bind(&product.name_en)
        .bind(&product.desc_pl)
        .bind(&product.desc_en)
        .bind(product.wood_qua)
        .bind(product.metal_qua)
        .bind(product.glass_qua)
        .bind(product.price)
        .bind(product.width)
        .bind(product.height)
        .bind(product.depth)
        .bind(product.id) // To ID z lokacji WHERE
        .execute(pool)
        .await?;

    Ok(())
}