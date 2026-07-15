use crate::product::Product;
use crate::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use sqlx::SqlitePool;

pub async fn handler_put_product_new(
    State(state): State<AppState>,
    Json(payload): Json<Product>,
) -> Result<StatusCode, (StatusCode, String)> {
    println!("Odebrano żądanie utworzenia produktu: {}", payload.name_id);

    // Wywołujemy funkcję SQL, błędy mapujemy na 500 Internal Server Error
    put_product_new(&state.db, &payload)
        .await
        .map_err(|e| {
            eprintln!("Błąd bazy danych przy tworzeniu produktu: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    // Zwracamy czysty status 201 Created
    Ok(StatusCode::CREATED)
}
pub async fn put_product_new(pool: &SqlitePool, new_product: &Product) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO products (
            name_id, name_pl, name_en, desc_pl, desc_en,
            wood_qua, metal_qua, glass_qua, price,
            width, height, depth
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#
    )
        .bind(&new_product.name_id)
        .bind(&new_product.name_pl)
        .bind(&new_product.name_en)
        .bind(&new_product.desc_pl)
        .bind(&new_product.desc_en)
        .bind(new_product.wood_qua)
        .bind(new_product.metal_qua)
        .bind(new_product.glass_qua)
        .bind(new_product.price)
        .bind(new_product.width)
        .bind(new_product.height)
        .bind(new_product.depth)
        .execute(pool)
        .await?;

    Ok(())

}