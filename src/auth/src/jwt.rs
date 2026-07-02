use crate::claims_thingy::claims::Claims;
use axum::http::{HeaderMap, StatusCode};
use jsonwebtoken::{decode, DecodingKey, Validation};
use login::user::User;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::OnceLock;

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

pub fn load_users_from_json() -> Vec<User> {
    let path = Path::new("src/api/login/usrs.json");
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => {
            println!("UWAGA: Nie znaleziono pliku usrs.json, zwracam pustą listę!\n {}", path.display() );
            return vec![];
        }
    };
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).unwrap_or_else(|_| vec![])
}

pub fn initialize_jwt_secret() {
    // losowy klucz przy każdym starcie
    #[cfg(not(debug_assertions))]
    {
        use rand::Rng;
        let mut dynamic_key = [0u8; 32];
        rand::rng().fill(&mut dynamic_key);

        JWT_SECRET.set(dynamic_key.to_vec()).expect("Błąd inicjalizacji klucza (debug)");
        println!("🔑 DEBUG: Wygenerowano losowy klucz JWT.");
    }

    // ładowany z konfiguracji
    #[cfg(debug_assertions)]
    {
        let secret = std::env::var("JWT_SECRET_KEY")
            .expect("W trybie release zmienna JWT_SECRET_KEY jest wymagana!");

        JWT_SECRET.set(secret.into_bytes()).expect("Błąd inicjalizacji klucza (release)");
        println!("🔑 RELEASE: Załadowano klucz JWT z konfiguracji.");
    }
}