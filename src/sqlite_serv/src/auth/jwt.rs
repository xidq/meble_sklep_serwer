use crate::auth::claims::Claims;
use axum::http::{HeaderMap, StatusCode};
use jsonwebtoken::{decode, DecodingKey, Validation};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::OnceLock;
use rand::RngExt;
use sqlx::{Row};
use sqlx::sqlite::SqlitePool;
use crate::user::{User, UserRola};

pub static JWT_SECRET: OnceLock<Vec<u8>> = OnceLock::new();


pub fn extract_and_verify_jwt(headers: &HeaderMap) -> Result<Claims, StatusCode> {

    let auth_header = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?; // Brak nagłówka err401


    if !auth_header.starts_with("Bearer ") {
        return Err(StatusCode::BAD_REQUEST); // Zły format nagłówka err400
    }

    let token = &auth_header[7..];

    // Dekodowanie i krypto ver podpisu i czasu ważności
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET.get().expect("Klucz JWT nie został zainicjalizowany!")),
        &Validation::default(),
    )
        .map(|data| data.claims)
        .map_err(|_| StatusCode::UNAUTHORIZED) // Jeśli podpis sfałszowany lub token wygasł err401
}

// pub fn load_users_from_json() -> Vec<User> {
//
//     let path = Path::new("../../../api/login/usrs.json");
//     let file = match File::open(path) {
//         Ok(f) => f,
//         Err(_) => {
//             println!("UWAGA: Nie znaleziono pliku usrs.json, zwracam pustą listę!\n {}", path.display() );
//             return vec![];
//         }
//     };
//     let reader = BufReader::new(file);
//     serde_json::from_reader(reader).unwrap_or_else(|_| vec![])
// }
// pub async fn get_user_by_username(pool: &SqlitePool, username: &str) -> Result<Option<User>, sqlx::Error> {
//     let row = sqlx::query("SELECT * FROM users WHERE username = ?")
//         .bind(username)
//         .fetch_optional(pool)
//         .await?;
//
//     if let Some(r) = row {
//         // Wyciągamy surowe dane tekstowe z kolumn bazy danych
//         let permission_str: String = r.get("permission");
//
//         // Mapujemy tekst z bazy z powrotem na Twój enum UsrPermit
//         let permission = match permission_str.as_str() {
//             "Admin" => UserRola::Admin,
//             "User" => UserRola::User,
//             _ => UserRola::Guest,
//         };
//
//         // SQLite domyślnie zwraca liczby jako i64.
//         // Jeśli w Twojej strukturze User.id to np. i32 lub u32, dodaj rzutowanie: r.get::<i64, _>("id") as u32
//         let user_id: i64 = r.get("id");
//         let valid_str: String = r.get("valid");
//         let is_valid = valid_str == "true" || valid_str == "1";
//
//         Ok(Some(User {
//             id: user_id,
//             username: "".to_string(),
//             name: r.get("username"),
//             email: None,
//             password_hash: r.get("password_hash"),
//             permission,
//             valid: is_valid,
//         }))
//     } else {
//         Ok(None) // Nie znaleziono użytkownika o takim loginie
//     }
// }

pub fn initialize_jwt_secret() {
    // losowy klucz przy każdym starcie
    #[cfg(not(debug_assertions))]
    {
        use rand::Rng;
        let mut dynamic_key = [0u8; 32];
        rand::rng().fill(&mut dynamic_key);

        JWT_SECRET.set(dynamic_key.to_vec()).expect("Błąd inicjalizacji klucza (release)");
        println!("🔑 RELEASE: Wygenerowano losowy klucz JWT.");
    }

    // ładowany z konfiguracji
    #[cfg(debug_assertions)]
    {
        let secret = std::env::var("JWT_SECRET_KEY")
            .expect("W trybie debug zmienna JWT_SECRET_KEY jest wymagana!");

        JWT_SECRET.set(secret.into_bytes()).expect("Błąd inicjalizacji klucza (debug)");
        println!("🔑 DEBUG: Załadowano klucz JWT z konfiguracji.");
    }
}