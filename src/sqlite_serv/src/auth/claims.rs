use crate::auth::jwt::{extract_and_verify_jwt, JWT_SECRET};
use axum::extract::FromRequestParts;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
// --- MODELE DANYCH ---

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64,         // ID użytkownika (Subject)
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

        let secret = JWT_SECRET.get().expect("Klucz JWT nie został zainicjalizowany!");

        decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_ref()), // Przekazujemy referencję do bajtów klucza  ----------- EDIT był
            &Validation::default(),
        )
            .map(|data| data.claims)
            .map_err(|_| StatusCode::UNAUTHORIZED)
    }
}
pub struct OptionalClaims(pub Option<Claims>);
impl<S> axum::extract::FromRequestParts<S> for OptionalClaims
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible; // Nigdy nie odrzucamy żądania

    async fn from_request_parts(parts: &mut axum::http::request::Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Próbujemy użyć Twojej istniejącej logiki dla Claims
        // Claims implementuje FromRequestParts, więc wywołujemy go bezpośrednio
        match Claims::from_request_parts(parts, state).await {
            Ok(claims) => Ok(OptionalClaims(Some(claims))),
            Err(_) => Ok(OptionalClaims(None)), // Jeśli błąd (brak/zły token), zwracamy None zamiast błędu
        }
    }
}

// #[async_trait]
// impl<S> FromRequestParts<S> for Option<Claims>
// where
//     S: Send + Sync,
// {
//     type Rejection = std::convert::Infallible;
//
//     async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
//         // Spróbuj pobrać i zdekodować token
//         let auth_header = parts.headers.get(AUTHORIZATION)?.to_str().ok()?;
//         if !auth_header.starts_with("Bearer ") { return Ok(None); }
//
//         let token = &auth_header[7..];
//         let secret = JWT_SECRET.get()?;
//
//         let claims = decode::<Claims>(token, &DecodingKey::from_secret(secret), &Validation::default())
//             .map(|data| data.claims)
//             .ok(); // Zwróci None, jeśli token jest nieważny lub sfałszowany
//
//         Ok(claims)
//     }
// }

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