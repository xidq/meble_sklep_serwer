use sqlx::SqlitePool;
use crate::model::Model;

pub async fn put_model_add_to_database(pool: &SqlitePool, new_product: &Model) -> Result<(), sqlx::Error> {

    // Zamieniamy BTreeMap na czysty tekst (JSON)
    let model_json = serde_json::to_string(&new_product.model)
        .map_err(|e| sqlx::Error::Protocol(format!("Błąd serializacji JSON: {}", e).into()))?;

    sqlx::query(
        r#"
        INSERT INTO models (
            product_id, texture_ao, model
        ) VALUES (?, ?, ?)
        "#
    )
        .bind(new_product.product_id)
        .bind(&new_product.texture_ao)
        .bind(model_json) // <-- Tutaj wlatuje nasz wygenerowany String z nieskończoną ilością LOD-ów
        .execute(pool)
        .await?;

    Ok(())
}