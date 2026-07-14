use crate::auth::sending_data::{files_send_to_server, json_send_to_server, RodzajeDanychJson};
use crate::foto::FotoData;
use crate::product::get::get_products_id_by_nameid;
use crate::sql::AppState;
use avif_image_handler::save::avif_match;
use avif_image_handler::wczytywanie::main_wczytywanie::wczytaj_pliki;
use axum::extract::{Multipart, Path, State};
use axum::Json;
use http::StatusCode;
use serde_json::json;
use sqlx::SqlitePool;
use std::collections::BTreeMap;
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;
use crate::foto::get::get_image_data_by_id;

pub async fn handler_image_upload_to_server(
    State(state): State<AppState>,
    Path(item_name_id): Path<String>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, String)> {

    println!("Rozpoczęto odbieranie zdjęć dla: {}", item_name_id);

    // 1. Pobieramy ID produktu z bazy
    let id = get_products_id_by_nameid(&item_name_id, &state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Błąd ID: {}", e)))?;

    // 2. Tworzymy folder queued DLA TEGO KONKRETNEGO PRODUKTU
    let queued_path = format!("src/api/products/{}/queued", item_name_id);
    tokio::fs::create_dir_all(&queued_path)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Błąd folderu: {}", e)))?;

    let mut saved_files: Vec<PathBuf> = Vec::new();

    // 3. Zapisujemy gołe pliki z frontu
    while let Ok(Some(field)) = multipart.next_field().await {
        let file_name = field.file_name().unwrap_or("unknown.jpg").to_string();
        let path = std::path::Path::new(&queued_path).join(&file_name);

        if let Ok(data) = field.bytes().await
            && let Ok(mut file) = tokio::fs::File::create(&path).await {
                if file.write_all(&data).await.is_ok() {
                    saved_files.push(path);
                }

        }
    }

    // 4. DAWANIE ZNAĆ (Odpalamy workera w tle tylko dla tych plików!)
    // Klonujemy pule bazy i ID, bo zadanie w tle żyje własnym życiem
    let db_pool = state.db.clone();
    let name_id_clone = item_name_id.clone();

    tokio::spawn(async move {
        let _ = background_image_processor(db_pool, id, name_id_clone, saved_files).await;
    });

    // 5. Natychmiastowa odpowiedź do klienta, nie czekamy na AVIF!
    Ok((
        StatusCode::ACCEPTED, // 202 - Przyjęto do przetwarzania
        Json(json!({ "message": "Zdjęcia odebrane. Trwa konwersja i przetwarzanie w tle..." })),
    ))
}

async fn background_image_processor(
    pool: sqlx::SqlitePool,
    product_id: i64,
    item_name_id: String,
    queued_files: Vec<PathBuf>,
) -> Result<(), String> {
    println!("Rozpoczęto konwersję w tle dla: {}", item_name_id);

    // Docelowy folder na przekonwertowane pliki
    let final_dir = PathBuf::from(format!("src/api/products/{}/images", item_name_id));
    let _ = tokio::fs::create_dir_all(&final_dir).await;

    // 1. KONWERSJA AVIF
    for path in &queued_files {
        // Tu używamy Twoich starych funkcji
        if let Ok((foto, nazwa_org)) = wczytaj_pliki(path.clone()) {
            let _ = avif_match(nazwa_org, foto, &final_dir).await;
        }
        // Sprzątamy - usuwamy oryginał z queued
        let _ = tokio::fs::remove_file(path).await;
    }

    // 2. OGARNIANIE BAZY DANYCH (Zbieranie do BTreeMap)
    let mut warianty_zdjec: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    let mut files_to_send = Vec::new();

    if let Ok(mut entries) = tokio::fs::read_dir(&final_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.is_file() {
                files_to_send.push(path.clone());

                let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                let path_str = path.to_string_lossy().to_string();

                // Odcinamy rozszerzenie (np. ".avif")
                let base_name = file_name.split('.').next().unwrap_or(&file_name);

                // Dzielimy nazwę po "_"
                // W twoim starym kodzie rozdzielczość była 4. członem, np: nameId_coś_wariant_512
                let parts: Vec<&str> = base_name.split('_').collect();

                // Ustalanie wariantu i rozdzielczości.
                // Dopasuj indeksy do swojego nazewnictwa! Poniżej bezpieczne zakładanie,
                // że wariant jest przedostatni, a rozdzielczość na samym końcu:
                let rozdzielczosc = if !parts.is_empty() {
                    parts.last().unwrap().to_string() // np. "512"
                } else {
                    "0".to_string()
                };

                let wariant = if parts.len() > 1 {
                    format!("var_{}", parts[parts.len() - 2]) // np. "var_1"
                } else {
                    "default".to_string()
                };

                // MAGICZNE ENTRY API - samo zarządza zagnieżdżeniami!
                warianty_zdjec
                    .entry(wariant)
                    .or_default() // Jeśli nie ma wariantu, utwórz pustą mapę
                    .insert(rozdzielczosc, path_str); // I wrzuć do niego rozdzielczość + ścieżkę
            }
        }
    }

    // Twoja super struktura gotowa do akcji!
    let foto_data = FotoData {
        product_id,
        warianty_zdjec,
    };

    // 3. UPSERT DO BAZY DANYCH
    if let Err(e) = images_upsert_in_database(&pool, &foto_data).await {
        eprintln!("Błąd zapisu zdjęć do bazy dla {}: {}", item_name_id, e);
    }

    // 4. WYSYŁKA NA SERWER (Używamy funkcji napisanej przy modelach!)
    // Założyłem, że nazwiesz uniwersalną funkcję z poprzedniego etapu `files_send_to_server`
    if let Err(e) = files_send_to_server(&files_to_send, &item_name_id).await {
        eprintln!("Błąd wysyłki plików graficznych na front dla {}: {}", item_name_id, e);
    } else {
        println!("Zakończono pełen cykl (Konwersja -> DB -> Frontend) dla {}", item_name_id);
    }

    // 2. Pobranie danych z bazy
    let model_data = get_image_data_by_id(product_id, &pool).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())).map_err(|e| format!("Błąd pobierania danych z bazy: {:?}", e))?;;

    // 3. Konwersja na JSON gotowy do wysyłki
    // Dzięki #[serde(flatten)], serde_json zamieni strukturę na płaski obiekt:
    // { "product_id": 1, "texture_ao": "...", "LOD0": "...", "LOD1": "..." }
    let json_payload = serde_json::to_value(&model_data)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())).map_err(|e| format!("Błąd serializacji JSON: {:?}", e))?;

    // 4. Wysyłka JSON-a (Twoja nowa funkcja)
    json_send_to_server(&item_name_id, json_payload, RodzajeDanychJson::ImgFront).await?;

    Ok(())
}

pub async fn images_upsert_in_database(pool: &SqlitePool, product: &FotoData) -> Result<(), sqlx::Error> {

    // Zwijamy BTreeMap do płaskiego JSON-a
    let images_json = serde_json::to_string(&product.warianty_zdjec)
        .map_err(|e| sqlx::Error::Protocol(format!("Błąd serializacji JSON: {}", e)))?;

    sqlx::query(
        r#"
        INSERT INTO images (product_id, warianty_zdjec)
        VALUES (?, ?)
        ON CONFLICT(product_id) DO UPDATE SET
            warianty_zdjec = excluded.warianty_zdjec
        "#
    )
        .bind(product.product_id)
        .bind(images_json)
        .execute(pool)
        .await?;

    Ok(())
}