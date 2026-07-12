use sqlx::SqlitePool;
use crate::model::Model;

pub async fn post_model_update_in_database(pool: &SqlitePool, updated_product: &Model) -> Result<(), sqlx::Error> {

    // Zamieniamy BTreeMap z powrotem na czysty tekst (JSON)
    let model_json = serde_json::to_string(&updated_product.model)
        .map_err(|e| sqlx::Error::Protocol(format!("Błąd serializacji JSON przy update: {}", e).into()))?;

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