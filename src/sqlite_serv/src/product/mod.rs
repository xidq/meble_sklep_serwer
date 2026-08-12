pub mod get;
pub mod post;
pub mod put;
pub mod delete;

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow, Debug, Clone)]
pub struct Product{
    pub id: i64,
    pub name_id: String,
    pub name_pl: String,
    pub name_en: String,
    pub desc_pl: String,
    pub desc_en: String,
    pub wood_qua: f32,
    pub metal_qua: f32,
    pub glass_qua: f32,
    pub plastik_qua: f32,
    pub price: f32,
    pub width: f32,
    pub height: f32,
    pub depth: f32,
    pub texture_scale: f32
}
#[derive(Deserialize, Debug, Clone)]
pub struct ProductUpdate {
    pub name_id: Option<String>,
    pub name_pl: Option<String>,
    pub name_en: Option<String>,
    pub desc_pl: Option<String>,
    pub desc_en: Option<String>,
    pub wood_qua: Option<f32>,
    pub metal_qua: Option<f32>,
    pub glass_qua: Option<f32>,
    pub price: Option<f32>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub depth: Option<f32>,
    pub plastik_qua: Option<f32>,
    pub texture_scale: Option<f32>,
}

impl Default for Product {
    fn default() -> Product {
        Self{
            id: 0,
            name_id: String::new(),
            name_pl: String::new(),
            name_en: String::new(),
            desc_pl: String::new(),
            desc_en: String::new(),
            wood_qua: 0.0,
            metal_qua: 0.0,
            glass_qua: 0.0,
            plastik_qua: 0.0,
            price: 0.0,
            width: 0.0,
            height: 0.0,
            depth: 0.0,
            texture_scale: 1.0,
        }
    }
}

impl Product{
    pub fn new(
        // name_id: impl Into<String>,
        // name_pl: impl Into<String>,
        // name_en: impl Into<String>,
        // desc_pl: impl Into<String>,
        // desc_en: impl Into<String>,
        // wood_qua: f32,
        // metal_qua: f32,
        // glass_qua: f32,
        // price: f32,
        // width: f32,
        // height: f32,
        // depth: f32,
    ) -> Self {
        // Self {
        //     id: 0,
        //     name_id: name_id.into(),
        //     name_pl: name_pl.into(),
        //     name_en: name_en.into(),
        //     desc_pl: desc_pl.into(),
        //     desc_en: desc_en.into(),
        //     wood_qua,
        //     metal_qua,
        //     glass_qua,
        //     price,
        //     width,
        //     height,
        //     depth,
        // }
        Self::default()
    }
    pub fn add_name_id(mut self, val:impl Into<String>) -> Self{
            self.name_id = val.into();
            self
    }
    pub fn add_name_pl(mut self, val:impl Into<String>) -> Self{
        self.name_pl = val.into();
        self
    }
    pub fn add_name_en(mut self, val:impl Into<String>) -> Self{
        self.name_en = val.into();
        self
    }
    pub fn add_desc_pl(mut self, val:impl Into<String>) -> Self{
        self.desc_pl = val.into();
        self
    }
    pub fn add_desc_en(mut self, val:impl Into<String>) -> Self{
        self.desc_en = val.into();
        self
    }
    pub fn add_wood_qua(mut self, val:f32) -> Self {
        self.wood_qua = val;
        self
    }
    pub fn add_metal_qua(mut self, val:f32) -> Self {
        self.metal_qua = val;
        self
    }
    pub fn add_glass_qua(mut self, val:f32) -> Self {
        self.glass_qua = val;
        self
    }
    pub fn add_plastik_qua(mut self, val:f32) -> Self {
        self.plastik_qua = val;
        self
    }
    pub fn add_price(mut self, val:f32) -> Self {
        self.price = val;
        self
    }
    pub fn add_width(mut self, val:f32) -> Self {
        self.width = val;
        self
    }
    pub fn add_height(mut self, val:f32) -> Self {
        self.height = val;
        self
    }
    pub fn add_depth(mut self, val:f32) -> Self {
        self.depth = val;
        self
    }
    pub fn add_texture_scale(mut self, val:f32) -> Self {
        self.texture_scale = val;
        self
    }

    pub fn update(&mut self, change: ProductUpdate) {
        if let Some(val) = change.name_id { self.name_id = val; }
        if let Some(val) = change.name_pl { self.name_pl = val; }
        if let Some(val) = change.name_en { self.name_en = val; }
        if let Some(val) = change.desc_pl { self.desc_pl = val; }
        if let Some(val) = change.desc_en { self.desc_en = val; }
        if let Some(val) = change.wood_qua { self.wood_qua = val; }
        if let Some(val) = change.metal_qua { self.metal_qua = val; }
        if let Some(val) = change.glass_qua { self.glass_qua = val; }
        if let Some(val) = change.plastik_qua { self.plastik_qua = val; }
        if let Some(val) = change.price { self.price = val; }
        if let Some(val) = change.width { self.width = val; }
        if let Some(val) = change.height { self.height = val; }
        if let Some(val) = change.depth { self.depth = val; }
        if let Some(val) = change.texture_scale { self.texture_scale = val; }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct ProductMapping {
    pub id: i64,
    pub name_id: String,
}