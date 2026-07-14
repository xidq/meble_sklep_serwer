use crate::model::Model;
use crate::product::get::get_products_id_by_nameid;
use crate::sql::AppState;
use axum::extract::{Multipart, State};
use axum::Json;
use http::StatusCode;
use serde_json::json;
use sqlx::SqlitePool;
use std::collections::BTreeMap;
use std::io::Read;
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;
use crate::auth::sending_data::{files_send_to_server, json_send_to_server, RodzajeDanychJson};
use crate::model::get::get_models_data_by_id;

pub async fn handler_model_upload_to_server(
    State(state): State<AppState>,
    axum::extract::Path(item_name_id): axum::extract::Path<String>, // Odczytujemy name_id z URL
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, String)> {

    println!("Rozpoczęto ogarnianie modeli dla: {}", item_name_id);

    // 1. Pobranie ID z obsługą błędu
    let id = get_products_id_by_nameid(&item_name_id, &state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Błąd pobierania ID przedmiotu z bazy: {}", e),
            )
        })?;

    let base_path = format!("src/api/products/{}/model", item_name_id);

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

    // 3. Zapisywanie plików
    while let Ok(Some(field)) = multipart.next_field().await {
        let file_name = field.file_name().unwrap_or("unknown").to_string();
        let path = std::path::Path::new(&base_path).join(&file_name);

        if let Ok(data) = field.bytes().await
            && let Ok(mut file) = tokio::fs::File::create(&path).await {
                if file.write_all(&data).await.is_ok() {
                    vec_sciezki_plikow.push(path);
                }

        }
    }

    // 4. Analiza zapisanych plików
    for path in &vec_sciezki_plikow {
        // Bezpieczne wyciągnięcie rozszerzenia i nazwy pliku
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

    // 5. Aktualizacja w bazie – UPSERT robimy RAZ po zebraniu wszystkich LOD-ów i tekstury!
    model_upsert_in_database(&state.db, &plik_modelu)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Błąd zapisu do bazy: {}", e)))?;

    // 6. Wysłanie plików na frontend server
    files_send_to_server(&vec_sciezki_plikow, &item_name_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Błąd wysyłania plików na frontend: {}", e)))?;
    // let dane_json = json!({ /* Twoja struktura var_1, var_2 itd. */ });
    // json_send_to_server(&item_name_id, dane_json, "img_front").await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Błąd wysyłania json na frontend: {}", e)))?;

    // 2. Pobranie danych z bazy
    let model_data = get_models_data_by_id(id, &state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 3. Konwersja na JSON gotowy do wysyłki
    // Dzięki #[serde(flatten)], serde_json zamieni strukturę na płaski obiekt:
    // { "product_id": 1, "texture_ao": "...", "LOD0": "...", "LOD1": "..." }
    let json_payload = serde_json::to_value(&model_data)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 4. Wysyłka JSON-a (Twoja nowa funkcja)
    json_send_to_server(&item_name_id, json_payload, RodzajeDanychJson::Models).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    
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