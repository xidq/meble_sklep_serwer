use crate::AppState;
use crate::user::{RegisterRequest, User};
use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use http::StatusCode;
use regex::Regex;
use sqlx::sqlite::SqliteOperation::Insert;

/// Create new user with login, password, 2nd password (double verification) and email
// pub async fn handler_user_new(
//     State(state): State<AppState>,
//     Json(payload): Json<RegisterRequest>,
// ) -> impl IntoResponse {
//     let username = payload.username.trim();
//     let password = payload.password;
//     let email = payload.email;
//     let email_ref = email.as_ref();
//     let name = payload.name;
//     let conditions_accept: bool = payload.registration_conditions;
//
//     // val loginu (Regex)
//     let username_regex = Regex::new(r"^[a-zA-Z0-9_]{3,20}$").unwrap();
//     if !username_regex.is_match(username) {
//         return (
//             StatusCode::BAD_REQUEST,
//             Json(serde_json::json!({
//                 "error": "Login musi mieć od 3 do 20 znaków i może zawierać tylko litery, cyfry oraz znak '_'"
//             }))
//         ).into_response();
//     }
//
//     if password != payload.confirm_password {
//         return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Hasła nie są identyczne"}))).into_response();
//     }
//
//     // email val (2nd verification)
//     if !email_ref.is_some_and(|cc| cc.contains("@")) || email_ref.is_some_and(|xx| xx.len() < 5) {
//         return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Niepoprawny format emaila"}))).into_response();
//     }
//     if !conditions_accept {
//         return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Niepoprawny format emaila"}))).into_response();
//     }
//
//
//     // pass val
//     if password.len() < 8 || password.len() > 100 {
//         return (
//             StatusCode::BAD_REQUEST,
//             Json(serde_json::json!({
//                 "error": "Hasło musi mieć od 8 do 100 znaków"
//             }))
//         ).into_response();
//     }
//
//     // pass hash
//     let new_user = match User::new(username, name, email.clone(), password) {
//         Ok(user) => user,
//         Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": "Błąd hashowania" }))).into_response(),
//     };
//
//     let permission_str = new_user.permission.to_string();
//     let valid_str = "false";
//
//
//     let insert_result = {
//         sqlx::query(
//         "
//         INSERT INTO users (username, password_hash, email, permission, valid) //email jest teraz w users_data
//         VALUES (?, ?, ?, ?, ?)
//         "
//     )
//         .bind(&new_user.username)
//         .bind(&new_user.password_hash)
//         .bind(&email)
//         .bind(permission_str)
//         .bind(valid_str)
//         .execute(&state.db)
//         .await
//
//         sqlx::query(
//             INSERT INTO users_data (email)
//             VALUES (?)
//         )
//
//     };
//
//     // obsługa wyniku z bazy
//     match insert_result {
//         Ok(_) => {
//             (StatusCode::CREATED, Json(serde_json::json!({ "message": "Konto utworzone pomyślnie!" }))).into_response()
//         }
//         Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
//             // Zamiast iterować po pętli (jak w JSON), SQLite samo rzuci błędem UNIQUE,
//             // jeśli ktoś spróbuje użyć zajętego loginu. To rozwiązanie jest odporne na tzw. race conditions.
//             (StatusCode::CONFLICT, Json(serde_json::json!({ "error": "Użytkownik o takiej nazwie już istnieje" }))).into_response()
//         }
//         Err(e) => {
//             eprintln!("Błąd zapisu do bazy: {:?}", e);
//             (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": "Wystąpił błąd serwera podczas tworzenia bazy" }))).into_response()
//         }
//     }
// }

pub async fn handler_user_new(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    let username = payload.username.trim();
    let password = payload.password;
    let email = payload.email;
    let email_ref = email.as_ref();
    let name: Option<String> = None;
    let surname: Option<String> = None;
    let conditions_accept: bool = payload.registration_conditions;

    // val loginu (Regex)
    let username_regex = Regex::new(r"^[a-zA-Z0-9_]{3,20}$").unwrap();
    if !username_regex.is_match(username) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Login musi mieć od 3 do 20 znaków i może zawierać tylko litery, cyfry oraz znak '_'"
            }))
        ).into_response();
    }

    if password != payload.confirm_password {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Hasła nie są identyczne"}))).into_response();
    }

    // email val
    if !email_ref.is_some_and(|cc| cc.contains("@")) || email_ref.is_some_and(|xx| xx.len() < 5) {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Niepoprawny format emaila"}))).into_response();
    }

    // Fixed error message for conditions
    if !conditions_accept {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Musisz zaakceptować warunki regulaminu"}))).into_response();
    }

    // pass val
    if password.len() < 8 || password.len() > 100 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Hasło musi mieć od 8 do 100 znaków"
            }))
        ).into_response();
    }

    // pass hash
    let new_user = match User::new(username, None, None, email.clone(), password) {
        Ok(user) => user,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": "Błąd hashowania" }))).into_response(),
    };

    let permission_str = new_user.permission.to_string();
    let valid_str = false; // boolean type fits better with BOOLEAN column

    // Use a transaction to safely write to both 'users' and 'users_data'
    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            eprintln!("Błąd rozpoczęcia transakcji: {:?}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": "Wystąpił błąd serwera" }))).into_response();
        }
    };

    // 1. Insert into users (email and name are removed from this table)
    let user_insert = sqlx::query(
        "
        INSERT INTO users (username, password_hash, permission, valid)
        VALUES (?, ?, ?, ?)
        "
    )
        .bind(&new_user.username)
        .bind(&new_user.password_hash)
        .bind(permission_str)
        .bind(valid_str)
        .execute(&mut *tx)
        .await;

    if let Err(e) = user_insert {
        return match e {
            sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
                (StatusCode::CONFLICT, Json(serde_json::json!({ "error": "Użytkownik o takiej nazwie już istnieje" }))).into_response()
            }
            _ => {
                eprintln!("Błąd zapisu użytkownika: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": "Wystąpił błąd serwera" }))).into_response()
            }
        };
    }

    // 2. Insert into users_data (username is PK, holds email and name)
    let data_insert = sqlx::query(
        "
        INSERT INTO users_data (username, email, name, surname)
        VALUES (?, ?, ?, ?)
        "
    )
        .bind(&new_user.username)
        .bind(&email)
        .bind(&name)
        .bind(&surname)
        .execute(&mut *tx)
        .await;

    if let Err(e) = data_insert {
        eprintln!("Błąd zapisu danych użytkownika: {:?}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": "Wystąpił błąd serwera podczas zapisu danych" }))).into_response();
    }

    // Commit transaction
    if let Err(e) = tx.commit().await {
        eprintln!("Błąd zatwierdzenia transakcji: {:?}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": "Wystąpił błąd serwera" }))).into_response();
    }

    (StatusCode::CREATED, Json(serde_json::json!({ "message": "Konto utworzone pomyślnie!" }))).into_response()
}