use crate::auth::claims::Claims;
use axum::http::{HeaderMap, StatusCode};
use jsonwebtoken::{decode, DecodingKey, Validation};
// use sqlx::Row;
use std::sync::OnceLock;

pub static JWT_SECRET: OnceLock<Vec<u8>> = OnceLock::new();


pub fn extract_and_verify_jwt(headers: &HeaderMap) -> Result<Claims, StatusCode> {
println!("Extracting JWT");
    let auth_header = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?; // Brak nagłówka err401

    println!("Auth header: {:?}", auth_header);
    if !auth_header.starts_with("Bearer ") {
        println!("auth_header.starts_with nie zaczyna sie Bearer");
        return Err(StatusCode::BAD_REQUEST); // Zły format nagłówka err400
    }

    let token = &auth_header[7..];

    println!("Auth token: {:?}", token);

    // Dekodowanie i krypto ver podpisu i czasu ważności
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET.get().expect("Klucz JWT nie został zainicjalizowany!")),
        &Validation::default(),
    )
        .map(|data| data.claims)
        .map_err(|_| StatusCode::UNAUTHORIZED) // Jeśli podpis sfałszowany lub token wygasł err401
}

/// Initializing JWT, different effect for dev and -r.
/// 
/// For debug|tests is taken from env file
pub fn initialize_jwt_secret() {
    // losowy klucz przy każdym starcie
    // #[cfg(not(debug_assertions))]
    // {
    //     use rand::Rng;
    //     let mut dynamic_key = [0u8; 32];
    //     rand::rng().fill(&mut dynamic_key);
    //
    //     JWT_SECRET.set(dynamic_key.to_vec()).expect("Błąd inicjalizacji klucza (release)");
    //     println!("RELEASE: Wygenerowano losowy klucz JWT.");
    // }
    //
    // // ładowany z konfiguracji
    // #[cfg(debug_assertions)]
    {
        let secret = std::env::var("JWT_SECRET_KEY")
            .expect("W trybie debug zmienna JWT_SECRET_KEY jest wymagana!");

        JWT_SECRET.set(secret.into_bytes()).expect("Błąd inicjalizacji klucza (debug)");
        println!("DEBUG: Załadowano klucz JWT z konfiguracji.");
    }
}