use crate::auth::sending_data::{files_send_to_server, json_send_to_server, RodzajeDanychJson};
use crate::foto::{get_items_prefix, FotoData};
use crate::model::{Model, ModelPayload};
use crate::product::get::get_products_nameid_by_id;
use crate::AppState;
use axum::extract::{Multipart, State};
use axum::Json;
use http::StatusCode;
use serde_json::json;
use sqlx::SqlitePool;
use std::collections::BTreeMap;
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;
// todo!("ogarnąć zbieranie i wysyłanie danych o modelach 3d (wood,metal,glass) oraz ogarnąć bazę danych modeli o te elementy ;)")
pub async fn handler_model_upload_to_server(
    State(state): State<AppState>,
    axum::extract::Path(idx): axum::extract::Path<String>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, String)> {

    println!("Rozpoczęto ogarnianie modeli dla: {}", idx);
    let id = idx.as_str().parse::<i64>().unwrap();
    let name_id = get_products_nameid_by_id(id, &state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Błąd pobierania ID przedmiotu z bazy: {}", e),
            )
        })?;

    let base_path = format!("{}/products/{}/model", get_items_prefix(), name_id);

    // 2. Upewniamy się, że folder istnieje (asynchronicznie)
    tokio::fs::create_dir_all(&base_path)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Błąd tworzenia folderu: {}", e)))?;

    let mut plik_modelu = Model {
        product_id: id,
        texture_ao: None,
        model: BTreeMap::new(),
    };

    // Zbieramy ścieżki jako PathBuf, żeby nie martwić się o lifetimes
    let mut vec_sciezki_plikow: Vec<PathBuf> = Vec::new();


    while let Ok(Some(field)) = multipart.next_field().await {
        let file_name = field.file_name().unwrap_or("unknown").to_string();
        let path = std::path::Path::new(&base_path).join(&file_name);

        if let Ok(data) = field.bytes().await
            && let Ok(mut file) = tokio::fs::File::create(&path).await
                && file.write_all(&data).await.is_ok() {
                    vec_sciezki_plikow.push(path);


        }
    }


    for path in &vec_sciezki_plikow {
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let path_str = path.to_string_lossy().to_string();

        match extension.as_str() {
            "dds" | "webp" | "avif" | "png" | "jpg" => {
                plik_modelu.texture_ao = Some(path_str);
            }
            "glb" => {
                // Szukamy członu zawierającego 'LOD' (wielkość liter ignorowana)
                // Np. nazwa "krzeslo_LOD1_wersja2.glb" -> wyciągnie "LOD1"
                let lod_lvl = file_name
                    .split('_')
                    .find(|part| part.to_uppercase().contains("LOD"))
                    .map(|s| s.split('.').next().unwrap_or(s).to_uppercase()) // Usuwamy kropkę z rozszerzeniem, jeśli LOD jest na końcu
                    .unwrap_or_else(|| "LOD0".to_string()); // Jeśli nie znajdzie LOD w nazwie, zakłada LOD0

                plik_modelu.model.insert(lod_lvl, path_str);
            }
            _ => {
                println!("Zignorowano plik o nieznanym rozszerzeniu: {}", file_name);
            }
        }
    }


    let payload = get_model_stats_by_nameid(&name_id, &state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Błąd pobierania danych z bazy: {:?}", e)))?;;

    // 5. Aktualizacja w bazie – UPSERT robimy RAZ po zebraniu wszystkich LOD-ów i tekstury!
    model_upsert_in_database(&state.db, &plik_modelu)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Błąd zapisu do bazy: {}", e)))?;

    // 6. Wysłanie plików na frontend server
    files_send_to_server(&vec_sciezki_plikow, &name_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Błąd wysyłania plików na frontend: {}", e)))?;

    let json_payload = serde_json::to_value(&payload)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Błąd serializacji JSON: {:?}", e)))?;


    json_send_to_server(&name_id, json_payload, RodzajeDanychJson::Models).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Błąd wysyłki JSON: {:?}", e)))?;
    // Zwracamy czysty, czytelny JSON o sukcesie
    Ok((
        StatusCode::OK,
        Json(json!({ "message": "Modele wgrane, zapisane i wysłane pomyślnie." })),
    ))
}

pub async fn model_upsert_in_database(pool: &SqlitePool, product: &Model) -> Result<(), sqlx::Error> {

    // Zamieniamy BTreeMap na JSON (tak samo jak wcześniej)
    let model_json = serde_json::to_string(&product.model)
        .map_err(|e| sqlx::Error::Protocol(format!("Błąd serializacji JSON: {}", e)))?;

    // Magia dzieje się w zapytaniu SQL:
    sqlx::query(
        r#"
        INSERT INTO models (product_id, texture_ao, model)
        VALUES (?, ?, ?)
        ON CONFLICT(product_id) DO UPDATE SET
            texture_ao = excluded.texture_ao,
            model = excluded.model
        "#
    )
        .bind(product.product_id)
        .bind(&product.texture_ao)
        .bind(model_json)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn get_model_stats_by_nameid(
    name_id: &str,
    pool: &SqlitePool,
) -> Result<ModelPayload, sqlx::Error>{
    let model = sqlx::query_as::<_, ModelPayload>(
        "SELECT
            name_id,
            wood_qua AS wood,
            metal_qua AS metal,
            glass_qua AS glass
        FROM products WHERE name_id = ?"
    )
        .bind(name_id)
        .fetch_one(pool)
        .await?;
    Ok(model)
}
