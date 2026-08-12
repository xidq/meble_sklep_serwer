use axum::extract::State;
use axum::Json;
use http::StatusCode;
use serde_json::json;
use sqlx::SqlitePool;
use crate::AppState;
use crate::auth::sending_data::{json_send_to_server, RodzajeDanychJson};
use crate::model::{Model, ModelPayload};

pub async fn post_model_update_in_database(pool: &SqlitePool, updated_product: &Model) -> Result<(), sqlx::Error> {

    // Zamieniamy BTreeMap z powrotem na czysty tekst (JSON)
    let model_json = serde_json::to_string(&updated_product.model)
        .map_err(|e| sqlx::Error::Protocol(format!("Błąd serializacji JSON przy update: {}", e)))?;

    // Wykonujemy UPDATE, szukając po product_id
    sqlx::query(
        r#"
        UPDATE models
        SET texture_ao = ?, model = ?
        WHERE product_id = ?
        "#
    )
        .bind(&updated_product.texture_ao)
        .bind(model_json)
        .bind(updated_product.product_id)
        .execute(pool)
        .await?;

    Ok(())
}

// #[axum::debug_handler]
pub async fn handler_refresh_model_json_at_front(
    State(state): State<AppState>,
    axum::extract::Path(idx): axum::extract::Path<String>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, String)> {

    println!("Rozpoczęto ogarnianie modeli dla: {}", idx);

    // Bezpieczne parsowanie ID zamiast unwrap()
    let id = idx.parse::<i64>().map_err(|_| {
        (StatusCode::BAD_REQUEST, "Niepoprawny format ID".to_string())
    })?;

    // Zapytanie z JOIN do tabeli models po texture_scale
    let model = sqlx::query_as::<_, ModelPayload>(
        "SELECT
            p.name_id,
            p.wood_qua AS wood,
            p.metal_qua AS metal,
            p.glass_qua AS glass,
            p.plastik_qua AS plastik,
            COALESCE(m.texture_scale, 1.0) AS texture_scale
        FROM products p
        LEFT JOIN models m ON p.id = m.product_id
        WHERE p.id = ?"
    )
        .bind(id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Błąd zbierania danych z bazy: {:?}", e)))?;

    let json_payload = serde_json::to_value(&model)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Błąd serializacji JSON: {:?}", e)))?;

    json_send_to_server(&model.name_id, json_payload, RodzajeDanychJson::Models).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Błąd wysyłki JSON: {:?}", e)))?;

    Ok((
        StatusCode::OK,
        Json(json!({ "message": "Dane modelu zczytane i wysłane pomyślnie." })),
    ))
}

pub async fn handler_refresh_all_models_json_at_front(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, String)> {

    let modele = sqlx::query_as::<_, ModelPayload>(
        "SELECT
            p.name_id,
            p.wood_qua AS wood,
            p.metal_qua AS metal,
            p.glass_qua AS glass,
            p.plastik_qua AS plastik,
            COALESCE(m.texture_scale, 1.0) AS texture_scale
        FROM products p
        LEFT JOIN models m ON p.id = m.product_id"
    )
        .fetch_all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Błąd zbierania danych z bazy: {:?}", e)))?;

    for model in modele {
        let json_payload = serde_json::to_value(&model)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Błąd serializacji JSON: {:?}", e)))?;

        json_send_to_server(&model.name_id, json_payload, RodzajeDanychJson::Models).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Błąd wysyłki JSON: {:?}", e)))?;
    }

    Ok((
        StatusCode::OK,
        Json(json!({ "message": "Dane wszystkich modeli zczytane i wysłane pomyślnie." })),
    ))
}