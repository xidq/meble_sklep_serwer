use std::fs::File;
use std::io::Write;
use std::path::Path;
use axum::extract::State;
use axum::Json;
use axum::response::IntoResponse;
use regex::Regex;
use axum::http::StatusCode;
use auth::jwt::load_users_from_json;
use login::user::{User, UsrPermit};
use sqlite_serv::sql::AppState;
use crate::requests::register::RegisterRequest;

// pub async fn register_handler(Json(payload): Json<RegisterRequest>) -> impl IntoResponse {
//     let username = payload.username.trim();
//     let password = payload.password;
//
//     // walidacja loginu, użycie Regex
//     // ^[a-zA-Z0-9_]{3,20}$ to jest: 3-20 znaków, tylko litery, cyfry i _
//     let username_regex = Regex::new(r"^[a-zA-Z0-9_]{3,20}$").unwrap();
//
//     if !username_regex.is_match(username) {
//         return (
//             StatusCode::BAD_REQUEST,
//             Json(serde_json::json!({
//                 "error": "Login musi mieć od 3 do 20 znaków i zawierać tylko litery, cyfry oraz znak '_'"
//             }))
//         ).into_response();
//     }
//
//     if password.len() < 8 || password.len() > 100 {
//         return (
//             StatusCode::BAD_REQUEST,
//             Json(serde_json::json!({
//                 "error": "Hasło musi mieć od 8 do 100 znaków"
//             }))
//         ).into_response();
//     }
//
//     let mut users = load_users_from_json();
//     if users.iter().any(|u| u.name == username) {
//         return (StatusCode::CONFLICT, Json(serde_json::json!({ "error": "Użytkownik o takiej nazwie już istnieje" }))).into_response();
//     }
//
//     let next_id = users.iter().map(|u| u.id).max().unwrap_or(0) + 1;
//     let new_user = match User::new(next_id, username.to_string(), UsrPermit::User, &password) {
//         Ok(user) => user,
//         Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": "Błąd hashowania" }))).into_response(),
//     };
//
//     users.push(new_user);
//     let path = Path::new("src/api/login/usrs.json");
//     if let Ok(mut file) = File::create(path) {
//         let json_data = serde_json::to_string_pretty(&users).unwrap();
//         if file.write_all(json_data.as_bytes()).is_ok() {
//             (StatusCode::CREATED, Json(serde_json::json!({ "message": "Konto utworzone pomyślnie!" }))).into_response()
//         } else {
//             (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": "Błąd zapisu pliku" }))).into_response()
//         }
//     } else {
//         (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": "Nie można otworzyć bazy" }))).into_response()
//     }
// }

pub async fn register_handler(
    State(state): State<AppState>, // <-- Dodajemy dostęp do stanu aplikacji (i puli db_usr)
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    let username = payload.username.trim();
    let password = payload.password;

    // 1. Walidacja loginu (Regex)
    let username_regex = Regex::new(r"^[a-zA-Z0-9_]{3,20}$").unwrap();
    if !username_regex.is_match(username) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Login musi mieć od 3 do 20 znaków i zawierać tylko litery, cyfry oraz znak '_'"
            }))
        ).into_response();
    }

    // 2. Walidacja hasła
    if password.len() < 8 || password.len() > 100 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Hasło musi mieć od 8 do 100 znaków"
            }))
        ).into_response();
    }

    // 3. Hashowanie hasła
    // Baza sama nadaje ID, więc do Twojej funkcji User::new przekazujemy dummy id (np. 0)
    let new_user = match User::new(0, username.to_string(), UsrPermit::User, &password) {
        Ok(user) => user,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": "Błąd hashowania" }))).into_response(),
    };

    // Zmieniamy Enum na tekst, żeby pasował do schematu bazy: permission TEXT, valid TEXT
    let permission_str = new_user.permission.to_string(); // Możesz też użyć new_user.permit.to_string(), jeśli zaimplementowałeś Display
    let valid_str = "false";      // Zgodnie z typem TEXT w kolumnie `valid`

    // 4. Wstawienie do bazy danych
    // Używamy db_usr ze stanu aplikacji
    let insert_result = sqlx::query(
        "
        INSERT INTO users (username, password_hash, permission, valid)
        VALUES (?, ?, ?, ?)
        "
    )
        .bind(&new_user.name)          // Podpinamy zmienne ręcznie za pomocą .bind()
        .bind(&new_user.password_hash)
        .bind(permission_str)
        .bind(valid_str)
        .execute(&state.db_usr)
        .await;

    // 5. Obsługa wyniku z bazy
    match insert_result {
        Ok(_) => {
            (StatusCode::CREATED, Json(serde_json::json!({ "message": "Konto utworzone pomyślnie!" }))).into_response()
        }
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
            // Zamiast iterować po pętli (jak w JSON), SQLite samo rzuci błędem UNIQUE,
            // jeśli ktoś spróbuje użyć zajętego loginu. To rozwiązanie jest odporne na tzw. race conditions.
            (StatusCode::CONFLICT, Json(serde_json::json!({ "error": "Użytkownik o takiej nazwie już istnieje" }))).into_response()
        }
        Err(e) => {
            eprintln!("Błąd zapisu do bazy: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": "Wystąpił błąd serwera podczas tworzenia bazy" }))).into_response()
        }
    }
}