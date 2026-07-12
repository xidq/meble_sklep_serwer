use sqlx::SqlitePool;
use crate::foto::FotoData;
use crate::model::Model;

pub async fn get_image_data_by_id(id: i64, pool: &SqlitePool) -> Result<FotoData, sqlx::Error> {

    let model = sqlx::query_as::<_, FotoData>("SELECT * FROM images WHERE product_id = ?")
        .bind(id)
        .fetch_one(pool)
        .await?;

    Ok(model)
}
