use crate::sql;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;
use sqlx::Type;
use sqlx::FromRow;

#[derive(Clone)]
pub struct AppState {
    pub tx: tokio::sync::mpsc::Sender<()>,
    pub db: sqlx::sqlite::SqlitePool,
}

#[derive(Debug, Clone, Copy, Type, PartialEq, Deserialize, Serialize)]
#[repr(i32)]
pub enum Rozdzielczosci {
    R16 = 16,
    R32 = 32,
    R64 = 64,
    R128 = 128,
    R256 = 256,
    R512 = 512,
    R1024 = 1024,
    R2048 = 2048,
}
impl Rozdzielczosci {
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            16 | 32 | 64 | 128 | 256 | 512 | 1024 | 2048 => {
                // Skoro użyłeś #[repr(i32)], możemy bezpiecznie rzutować
                Some(unsafe { std::mem::transmute::<i32, sql::Rozdzielczosci>(value) })
            },
            _ => None,
        }
    }
}

#[derive(Debug, FromRow, Deserialize, Serialize)]
pub struct Product {
    pub id: i64,
    pub name_id: String,
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


#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct ImageData {
    pub resolution: Rozdzielczosci, // enum mapowany na INTEGER w bazie
    pub path: String,
}

// JEDEN STRUKT z wszystkimi danymi
#[derive(Debug, Serialize, Deserialize)]
pub struct ProductWithImages {
    pub product: Product,
    pub images: Vec<ImageData>,
}

pub async fn list_product_ids_and_names(pool: &SqlitePool) -> Result<Vec<(u64, String)>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (u64, String)>(
        "SELECT id, name FROM products"
    )
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn get_id_and_name_id(pool: &SqlitePool) -> Result<Vec<(i64, String)>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (i64, String)>(
        "SELECT id, name_id FROM products"
    )
        .fetch_all(pool)
        .await?;
    Ok(rows)
}


// Pobiera produkt z jego obrazkami (wszystkie rozdzielczości)
pub async fn get_product_with_images(
    product_id: i64,
    pool: &SqlitePool,
) -> Result<ProductWithImages, sqlx::Error> {
    let product: Product = sqlx::query_as(
        "SELECT id, name, price, vat, description_pl, description_en,
                model_3d, texture_ao, texture_normal,
                width, height, depth
         FROM products
         WHERE id = ?"
    )
        .bind(product_id)
        .fetch_one(pool)
        .await?;

    let images: Vec<ImageData> = sqlx::query_as(
        "SELECT resolution, path
         FROM product_images
         WHERE product_id = ?"
    )
        .bind(product_id)
        .fetch_all(pool)
        .await?;

    Ok(ProductWithImages { product, images })
}

// Lista WSZYSTKICH produktów z ich obrazkami
pub async fn list_products_with_images(
    pool: &SqlitePool,
) -> Result<Vec<ProductWithImages>, sqlx::Error> {
    let products: Vec<Product> = sqlx::query_as(
        "SELECT id, name, price, vat, description_pl, description_en,
                model_3d, texture_ao, texture_normal,
                width, height, depth
         FROM products"
    )
        .fetch_all(pool)
        .await?;

    let mut result = Vec::with_capacity(products.len());
    for product in products {
        let images: Vec<ImageData> = sqlx::query_as(
            "SELECT resolution, path
             FROM product_images
             WHERE product_id = ?"
        )
            .bind(product.id)
            .fetch_all(pool)
            .await?;
        result.push(ProductWithImages { product, images });
    }
    Ok(result)
}

pub async fn insert_product(
    pool: &SqlitePool,
    data: Product,
    images: Vec<(Rozdzielczosci, String)>,
) -> Result<i64, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let product_id = sqlx::query(
        "INSERT INTO products (name, price, vat, description_pl, description_en,
                               model_3d, texture_ao, texture_normal,
                               width, height, depth)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
        .bind(data.name)
        .bind(data.price)
        .bind(data.vat)
        .bind(data.description_pl)
        .bind(data.description_en)
        .bind(data.model_3d)
        .bind(data.texture_ao)
        .bind(data.texture_normal)
        .bind(data.width)
        .bind(data.height)
        .bind(data.depth)
        .execute(&mut *tx)
        .await?
        .last_insert_rowid() as i64;

    for (resolution, path) in images {
        sqlx::query(
            "INSERT INTO product_images (product_id, resolution, path) VALUES (?, ?, ?)"
        )
            .bind(product_id)
            .bind(resolution as u32) // enum konwertuje się na u32
            .bind(path)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;
    Ok(product_id)
}
pub async fn update_product(
    pool: &SqlitePool,
    id: i64,
    data: &Product,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE products
         SET name = ?, price = ?, vat = ?, description_pl = ?, description_en = ?,
             model_3d = ?, texture_ao = ?, texture_normal = ?,
             width = ?, height = ?, depth = ?
         WHERE id = ?"
    )
        .bind(&data.name)
        .bind(&data.name_id)
        .bind(data.price)
        .bind(data.vat)
        .bind(&data.description_pl)
        .bind(&data.description_en)
        .bind(&data.model_3d)
        .bind(&data.texture_ao)
        .bind(&data.texture_normal)
        .bind(data.width)
        .bind(data.height)
        .bind(data.depth)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn insert_product_image(
    pool: &SqlitePool,
    product_id: i64,
    resolution: Rozdzielczosci,
    path: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO product_images (product_id, resolution, path)
         VALUES (?, ?, ?)
         ON CONFLICT(product_id, resolution)
         DO UPDATE SET path = excluded.path"
    )
        .bind(product_id) // SQLite używa i64 dla INTEGER
        .bind(resolution as i32)
        .bind(path)
        .execute(pool)
        .await?;
    Ok(())
}
pub async fn delete_product_image(
    pool: &SqlitePool,
    product_id: i64,
    resolution: Rozdzielczosci,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "DELETE FROM product_images WHERE product_id = ? AND resolution = ?"
    )
        .bind(product_id)
        .bind(resolution as i32)
        .execute(pool)
        .await?;
    Ok(())
}
pub async fn delete_product(pool: &SqlitePool, product_id: i64) -> Result<(), sqlx::Error> {
    // Jeśli masz ON DELETE CASCADE w bazie, wystarczy usunąć produkt
    sqlx::query("DELETE FROM products WHERE id = ?")
        .bind(product_id)
        .execute(pool)
        .await?;
    Ok(())
}