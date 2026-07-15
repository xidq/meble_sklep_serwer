use crate::model::Model;
use crate::AppState;
use axum::extract::State;
use axum::Json;
use http::StatusCode;
use sqlx::SqlitePool;

pub async fn handler_get_models_list(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<Vec<Model>>), (StatusCode, String)> {
    println!("Odebrano żądanie get_models_list");

    let models = get_models_list(&state.db)
        .await
        .map_err(|e| {
            eprintln!("Błąd bazy danych przy pobieraniu listy: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    // Jeśli wszystko poszło dobrze, zwracamy status 200 i listę zapakowaną w JSON
    Ok((StatusCode::OK, Json(models)))
}
pub async fn get_models_list(pool: &SqlitePool) -> Result<Vec<Model>, sqlx::Error> {

    let models = sqlx::query_as::<_, Model>("SELECT * FROM models")
        .fetch_all(pool)
        .await?;

    Ok(models)
}
pub async fn handler_get_models_data_by_id(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Result<(StatusCode, Json<Model>), (StatusCode, String)> {
    println!("Odebrano żądanie get_models_list");

    // Wywołujemy czystą funkcję SQL
    let models = get_models_data_by_id(id, &state.db)
        .await
        .map_err(|e| {
            eprintln!("Błąd bazy danych przy pobieraniu listy: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    // Jeśli wszystko poszło dobrze, zwracamy status 200 i listę zapakowaną w JSON
    Ok((StatusCode::OK, Json(models)))
}
pub async fn get_models_data_by_id(id: i64, pool: &SqlitePool) -> Result<Model, sqlx::Error> {

    let model = sqlx::query_as::<_, Model>("SELECT * FROM models WHERE product_id = ?")
        .bind(id)
        .fetch_one(pool)
        .await?;

    Ok(model)
}
// pub async fn get_models_nameid_by_id(id: i64, pool: &SqlitePool) -> Result<String, sqlx::Error> {
//
//     let model = sqlx::query_scalar::<_, String>("SELECT name_id FROM models WHERE id = ?")
//         .bind(id)
//         .fetch_one(pool)
//         .await?;
//
//     Ok(model)
// }

// pub async fn get_models_nameids_and_ids(pool: &SqlitePool) -> Result<Vec<ModelMapping>, sqlx::Error> {
//
//     let rows = sqlx::query("SELECT id, name_id FROM models")
//         .fetch_all(pool)
//         .await?;
//     let list: Vec<ModelMapping> = rows
//         .into_iter()
//         .map(|row| ModelMapping {
//             id: row.get("id"),
//             name_id: row.get("name_id"),
//         })
//         .collect();
//
//     Ok(list)
// }