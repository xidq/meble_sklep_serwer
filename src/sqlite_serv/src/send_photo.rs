use axum::Json;
use axum::response::IntoResponse;
use serde::Serialize;

#[derive(Serialize)]
struct PhotoInfo {
    file_name: String,
    source_url: String, // Skąd Node.js ma pobrać zdjęcie
}

#[derive(Serialize)]
struct ImageBatchResponse {
    command: String,
    target_path_on_frontend: String, // Ścieżka, gdzie Node.js ma to zapisać u siebie
    photos: Vec<PhotoInfo>,
}

pub async fn get_photos_handler() -> impl IntoResponse {
    let response = ImageBatchResponse {
        command: "SAVE_GALLERY_PHOTOS".to_string(),
        target_path_on_frontend: "./public/uploads/galleries/user_123/".to_string(),
        photos: vec![
            PhotoInfo {
                file_name: "wakacje_1.jpg".to_string(),
                source_url: "http://127.0.0.1:8080/static/images/img_9874.jpg".to_string(),
            },
            PhotoInfo {
                file_name: "wakacje_2.jpg".to_string(),
                source_url: "http://127.0.0.1:8080/static/images/img_9875.jpg".to_string(),
            },
        ],
    };

    Json(response)
}