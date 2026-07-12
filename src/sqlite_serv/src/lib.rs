extern crate core;

use std::sync::OnceLock;

pub mod sql;
// pub mod handler;
// pub mod sql_image_handling;
mod commands;
mod send_photo;
// mod sql_image_handling;
// pub mod image_handling;
pub mod sql_products;
pub mod user;
pub mod product;
pub mod foto;
pub mod model;
pub mod zamowienia;
pub mod auth;

// pub mod sql_image_handling;
pub static PEPPER_KEY: OnceLock<String> = OnceLock::new();