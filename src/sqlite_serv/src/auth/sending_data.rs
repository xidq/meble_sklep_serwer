use reqwest::multipart::{Form, Part};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Serialize)]
pub enum RodzajeDanychJson{
    Models,
    ImgFront
}

impl RodzajeDanychJson{
    pub fn str(&self) -> &'static str{
        match self{
            RodzajeDanychJson::Models => "models",
            RodzajeDanychJson::ImgFront => "img"
        }
    }
}
pub async fn files_send_to_server(
    sciezki_plikow: &[PathBuf],
    modyfikator: &str, // np. name_id
) -> Result<(), String> {

    // let client = reqwest::Client::new();
    let client = reqwest::Client::builder()
        .tls_danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| format!("Błąd budowania klienta: {}", e))?;
    let frontend_server = std::env::var("FRONTEND_SERVER")
        .unwrap_or_else(|_| "https://localhost:8444/".to_string());

    let serwer_adres = format!("{}api/produkty/{}", frontend_server, modyfikator);
    println!("będę wysyłał na adres: {}", serwer_adres);
    // 1. Inicjujemy pusty formularz multipart
    let mut form = Form::new();

    for path in sciezki_plikow {
        let bufor = tokio::fs::read(&path)
            .await
            .map_err(|e| format!("Błąd odczytu pliku: {}", e))?;

        let czysta_nazwa_pliku = path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown_file").to_string();
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();

        // 2. Dodany DDS i inne formaty
        let content_type = match extension.as_str() {
            "glb" => "model/gltf-binary",
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "webp" => "image/webp",
            "avif" => "image/avif",
            "dds" => "image/vnd-ms.dds", // <-- Obsługa DDS
            _ => "application/octet-stream",
        };

        // 3. Budujemy pojedynczą "część" (Part) dla konkretnego pliku
        let part = Part::bytes(bufor)
            .file_name(czysta_nazwa_pliku.clone())
            .mime_str(content_type)
            .map_err(|e| format!("Błąd parsowania MIME: {}", e))?;

        // 4. Doklejamy plik do formularza pod kluczem "files"
        // Twój serwer frontendowy będzie musiał odczytać to pole jako tablicę plików
        form = form.part("files", part);
    }

    // 5. Wysyłamy całą paczkę JEDNYM STRZAŁEM
    let response = client
        .post(&serwer_adres)
        .multipart(form) // <-- Zamiast surowego body, podpinamy przygotowany formularz
        .send()
        .await
        .map_err(|e| format!("Błąd żądania sieciowego: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Frontend server zwrócił błąd {} podczas odbierania paczki",
            response.status()
        ));
    }

    println!("[Rust] Pomyślnie wysłano paczkę {} plików do serwera (jedno zapytanie)", sciezki_plikow.len());

    Ok(())
}
pub async fn json_send_to_server(
    modyfikator: &str,
    json_data: serde_json::Value,
    typ: RodzajeDanychJson, // "models" lub "img_front"
) -> Result<(), String> {
    // let client = reqwest::Client::new();
    let client = reqwest::Client::builder()
        .tls_danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| format!("Błąd budowania klienta: {}", e))?;
    let frontend_server = std::env::var("FRONTEND_SERVER")
        .unwrap_or_else(|_| "https://localhost:8444/".to_string());

    // Używamy endpointu, który w Node.js obsługuje JSON-a i wywołuje rebuildRouterJson()
    let serwer_adres = format!("{}api/upload/json/{}/{}", frontend_server, typ.str(), modyfikator);
    println!("będę wysyłał na adres: {}", serwer_adres);

    let response = client
        .post(&serwer_adres)
        .json(&json_data)
        .send()
        .await
        .map_err(|e| format!("Błąd wysyłki JSON: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Serwer zwrócił błąd przy wysyłce JSON: {}", response.status()));
    }

    Ok(())
}