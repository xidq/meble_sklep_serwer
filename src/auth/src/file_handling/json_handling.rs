use std::fs::File;
use std::io::Write;
use axum::http::StatusCode;
use axum::Json;
use axum::response::IntoResponse;
use serde::Serialize;

pub fn file_serialisation<T>(path: &str, file_opened: Vec<T>) -> axum_core::response::Response
where T: Serialize{
    match File::create(path) {
        Ok(mut file) => {
            let json_data = serde_json::to_string_pretty(&file_opened).unwrap();
            if file.write_all(json_data.as_bytes()).is_ok() {
                (StatusCode::OK, Json(file_opened)).into_response()
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, "Błąd podczas zapisu do pliku json").into_response()
            }
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Nie można otworzyć pliku do zapisu").into_response(),
    }
}