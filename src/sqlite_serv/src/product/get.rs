use crate::product::{Product, ProductMapping};
use crate::sql::AppState;
use axum::extract::State;
use axum::Json;
use http::StatusCode;
use sqlx::{Row, SqlitePool};


pub async fn handler_get_products_list(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<Vec<Product>>), (StatusCode, String)> {
    println!("Odebrano żądanie get_products_list");

    // Wywołujemy czystą funkcję SQL
    let products = get_products_list(&state.db)
        .await
        .map_err(|e| {
            eprintln!("Błąd bazy danych przy pobieraniu listy: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    // Jeśli wszystko poszło dobrze, zwracamy status 200 i listę zapakowaną w JSON
    Ok((StatusCode::OK, Json(products)))
}
pub async fn get_products_list(pool: &SqlitePool) -> Result<Vec<Product>, sqlx::Error> {
    let products = sqlx::query_as::<_, Product>("SELECT * FROM products")
        .fetch_all(pool)
        .await?;

    Ok(products)
}
pub async fn handler_get_products_data_by_id(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Result<(StatusCode, Json<Product>), (StatusCode, String)> {
    println!("Odebrano żądanie get_products_list");

    // Wywołujemy czystą funkcję SQL
    let products = get_products_data_by_id(id, &state.db)
        .await
        .map_err(|e| {
            eprintln!("Błąd bazy danych przy pobieraniu listy: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    // Jeśli wszystko poszło dobrze, zwracamy status 200 i listę zapakowaną w JSON
    Ok((StatusCode::OK, Json(products)))
}

pub async fn handler_get_products_data_by_nameid(
    State(state): State<AppState>,
    axum::extract::Path(name_id): axum::extract::Path<String>,
) -> Result<(StatusCode, Json<Product>), (StatusCode, String)> {
    println!("Odebrano żądanie get_products_list");
    let id = get_products_id_by_nameid(&name_id, &state.db).await.ok().unwrap_or(0);
    // Wywołujemy czystą funkcję SQL1
    let products = get_products_data_by_id(id, &state.db)
        .await
        .map_err(|e| {
            eprintln!("Błąd bazy danych przy pobieraniu listy: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    // Jeśli wszystko poszło dobrze, zwracamy status 200 i listę zapakowaną w JSON
    Ok((StatusCode::OK, Json(products)))
}
pub async fn get_products_data_by_id(id: i64, pool: &SqlitePool) -> Result<Product, sqlx::Error> {

    let product = sqlx::query_as::<_, Product>("SELECT * FROM products WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await?;

    Ok(product)
}
pub async fn get_products_nameid_by_id(id: i64, pool: &SqlitePool) -> Result<String, sqlx::Error> {

    let product = sqlx::query_scalar::<_, String>("SELECT name_id FROM products WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await?;

    Ok(product)
}
pub async fn get_products_id_by_nameid(name_id: &str, pool: &SqlitePool) -> Result<i64, sqlx::Error> {

    let product = sqlx::query_scalar::<_, i64>("SELECT id FROM products WHERE name_id = ?")
        .bind(name_id)
        .fetch_one(pool)
        .await?;

    Ok(product)
}

pub async fn get_products_nameids_and_ids(pool: &SqlitePool) -> Result<Vec<ProductMapping>, sqlx::Error> {

    let rows = sqlx::query("SELECT id, name_id FROM products")
        .fetch_all(pool)
        .await?;
    let list: Vec<ProductMapping> = rows
        .into_iter()
        .map(|row| ProductMapping {
            id: row.get("id"),
            name_id: row.get("name_id"),
        })
        .collect();

    Ok(list)
}