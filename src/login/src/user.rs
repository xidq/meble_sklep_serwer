use bcrypt::{hash, verify};
use hmac::{KeyInit, Mac, SimpleHmac};
use serde::{Deserialize, Serialize};
use sha2::Sha512;
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
type HmacSha512 = SimpleHmac<Sha512>;


fn pepper_password(plain_password: &str, pepper: &[u8]) -> String {
    println!("pepper passowrd start!!!");

    let mut mac = HmacSha512::new_from_slice(pepper)
        .expect("SimpleHmac bez problemu poradzi sobie z każdą długością klucza");

    mac.update(plain_password.as_bytes());
    let result = mac.finalize();

    let bbb = hex::encode(result.into_bytes());
    println!("pepper passowrd end!!!");
    bbb
}

fn pepper_key() -> String {
    println!("pepper key!!!");
    std::env::var("PEPPER_KEY").expect("Brak PEPPER_KEY w .env")
}

impl User {
    // nowy użytkownik i auto hash hasła, hash ma już salt ;)
    pub fn new(id: i64, name: String, permission: UsrPermit, plain_password: &str, pepper: &str) -> Result<Self, bcrypt::BcryptError> {

        let peppered = pepper_password(plain_password, pepper.as_bytes());
        // DEFAULT_COST - 12 - optymalna siła hash, anty brute-force, MAX_COST - 31, MIN_COST - 4
        let password_hash = hash(&peppered, 12)?;

        Ok(User {
            id,
            name,
            permission,
            password_hash,
            valid: false,
        })
    }
    pub fn verify_password(&self, plain_password: &str, pepper: &str) -> bool {
        let peppered = pepper_password(plain_password, pepper.as_bytes());
        verify(&peppered, &self.password_hash).unwrap_or_default()
    }
}

