use axum::extract::FromRequestParts;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};



use crate::jwt::{extract_and_verify_jwt, JWT_SECRET};
// --- MODELE DANYCH ---

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: u64,         // ID użytkownika (Subject)
    pub username: String, // Nick
    pub role: String,     // Rola, np. "Admin" lub "User"
    pub exp: i64,         // Kiedy token wygasa (Timestamp)
}
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = StatusCode;
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts.headers
            .get(AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .ok_or(StatusCode::UNAUTHORIZED)?;

        if !auth_header.starts_with("Bearer ") {
            return Err(StatusCode::BAD_REQUEST);
        }

        let token = &auth_header[7..];

        // let secret = JWT_SECRET.get(/* index */).ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
        let secret = JWT_SECRET.get(/* index */).expect("Klucz JWT nie został zainicjalizowany!");

        decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret),
            &Validation::default(),
        )
            .map(|data| data.claims)
            .map_err(|_| StatusCode::UNAUTHORIZED)
    }
}

pub fn claims_match(headers: &HeaderMap) -> Result<Claims, Response> {
    match extract_and_verify_jwt(headers) {
        Ok(c) => {
            if c.role != "Admin" {
                return Err((StatusCode::FORBIDDEN, "Brak uprawnień administratora. Wymagana rola: Admin").into_response());
            }
            Ok(c)
        }
        Err(status) => Err(status.into_response()),
    }
}