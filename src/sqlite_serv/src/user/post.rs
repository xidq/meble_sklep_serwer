use crate::sql::AppState;
use crate::user::{RegisterRequest, User};
use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use http::StatusCode;
use regex::Regex;

pub async fn handler_user_new(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    let username = payload.username.trim();
    let password = payload.password;
    let email = payload.email;
    let email_ref = email.as_ref();
    let name = payload.name;

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

    if password != payload.confirm_password {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Hasła nie są identyczne"}))).into_response();
    }

    // 2. Walidacja formatu emaila
    if !email_ref.is_some_and(|cc| cc.contains("@")) || email_ref.is_some_and(|xx| xx.len() < 5) {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Niepoprawny format emaila"}))).into_response();
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
    let new_user = match User::new(username, name, email.clone(), password) {
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
        INSERT INTO users (username, password_hash, email, permission, valid)
        VALUES (?, ?, ?, ?, ?)
        "
    )
        .bind(&new_user.username)          // Podpinamy zmienne ręcznie za pomocą .bind()
        .bind(&new_user.password_hash)
        .bind(&email)
        .bind(permission_str)
        .bind(valid_str)
        .execute(&state.db)
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