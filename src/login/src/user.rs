use bcrypt::{hash, verify, DEFAULT_COST};
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Display)]
pub enum UsrPermit {
    User,
    Admin,
    Guest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub permission: UsrPermit,
    pub valid: bool,
    pub password_hash: String,
}

impl User {
    // nowy użytkownik i auto hash hasła
    pub fn new(id: i64, name: String, permission: UsrPermit, plain_password: &str) -> Result<Self, bcrypt::BcryptError> {
        // DEFAULT_COST (12) - optymalna siła hash, anty brute-force
        let password_hash = hash(plain_password, DEFAULT_COST)?;

        Ok(User {
            id,
            name,
            permission,
            password_hash,
            valid: false,
        })
    }
    pub fn verify_password(&self, plain_password: &str) -> bool {
        verify(plain_password, &self.password_hash).unwrap_or_default()
    }
}

