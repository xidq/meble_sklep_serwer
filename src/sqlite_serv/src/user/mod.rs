pub mod get;
pub mod put;
pub mod post;
pub mod delete;

use bcrypt::{hash, verify};
use hmac::{KeyInit, Mac, SimpleHmac};
use serde::{Deserialize, Deserializer, Serialize};
use sha2::Sha512;
use sqlx::FromRow;
use strum::Display;
use crate::PEPPER_KEY;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub confirm_password: String,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub email: Option<String>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub name: Option<String>
}
fn empty_string_as_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    Ok(opt.filter(|s| !s.trim().is_empty()))
}
#[derive(Clone, Serialize, Deserialize, FromRow, Debug)]
pub struct User{
    pub id: i64,
    pub username: String,
    pub name: Option<String>,
    pub email: Option<String>,

    #[serde(skip_serializing)]
    pub password_hash: String, //hash z salt i pepper itd...

    pub permission: UserRola,
    pub valid: bool,
}
#[derive(Serialize, Deserialize, Debug, Clone, Display, PartialEq)]
#[derive(sqlx::Type)] // <-- To naprawia błędy E0277!
#[sqlx(rename_all = "PascalCase")] // SQLite będzie widzieć wartości jako: "Admin", "User", "Guest"
pub enum UserRola{
    Admin,
    User,
    Guest
}
type HmacSha512 = SimpleHmac<Sha512>;

pub fn match_role(string: &str) -> UserRola{
    match string.to_lowercase().as_str() {
        "admin" => UserRola::Admin,
        "user" => UserRola::User,
        _ => UserRola::Guest
    }
}

fn pepper_password(plain_password: &str) -> String {
    println!("pepper passoword start!!!");

    let mut mac = HmacSha512::new_from_slice(get_pepper_key())
        .expect("SimpleHmac bez problemu poradzi sobie z każdą długością klucza");

    mac.update(plain_password.as_bytes());
    let result = mac.finalize();

    let bbb = hex::encode(result.into_bytes());
    println!("pepper passowrd end!!!");
    bbb
}

pub fn get_pepper_key() -> &'static [u8] {
    println!("pepper key!!!");
    PEPPER_KEY.get().expect("PEPPER_KEY nie jest zainicjalizowany").as_bytes()
    // std::env::var("PEPPER_KEY").expect("Brak PEPPER_KEY w .env")
}
impl User {
    pub fn new(
        username: impl Into<String>,
        name: Option<String>,
        email: Option<String>,
        password: impl Into<String>,
    ) -> Result<Self, bcrypt::BcryptError> {
        
        let peppered = pepper_password(&password.into());
        // DEFAULT_COST - 12 - optymalna siła hash, anty brute-force, MAX_COST - 31, MIN_COST - 4
        let password_hash = hash(&peppered, 12)?;

        Ok(Self {
            id: 0, // Baza danych nadpisze to swoim SERIAL/AUTOINCREMENT przy insercie
            username: username.into(),
            name,
            email,
            password_hash,
            permission: UserRola::User,
            valid: false,
        })
    }

    pub fn verify_password(&self, plain_password: &str) -> bool {
        let peppered = pepper_password(plain_password);
        verify(&peppered, &self.password_hash).unwrap_or_default()
    }
}