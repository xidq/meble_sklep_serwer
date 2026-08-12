pub mod get;
pub mod post;
pub mod put;
// mod ksef;

use chrono::{Datelike, Local};
use rust_decimal::{dec, Decimal};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqliteRow;
use sqlx::{FromRow, Row, SqlitePool};
use std::str::FromStr;
use strum::Display;
fn get_decimal(row: &SqliteRow, col: &str) -> Decimal {
    row.try_get::<String, _>(col)
        .ok()
        .and_then(|s| Decimal::from_str(&s).ok())
        .unwrap_or_default()
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CaloscioweZamowienie {
    dane: Zamowienie,
    przedmioty: Vec<ZamowieniePozycja>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum Waluta{
    Pln,
    Eur,
    Dol,
}
impl Waluta{
    pub fn get_multi(&self) -> Decimal {
        match self {
            Self::Pln => dec!(1.0),
            Self::Eur => dec!(4.30),
            Self::Dol => dec!(3.97),
        }
    }
    pub fn get_name(&self) -> &'static str {
        match self {
            Self::Pln => "pln",
            Self::Eur => "eur",
            Self::Dol => "dol",
        }
    }
}
#[derive(Serialize, Deserialize, Debug, Clone, Display, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
pub enum StatusOplacenia {
    Nieoplacone,
    Czesciowo,
    Oplacone,
    Zwrot,
}
#[derive(Serialize, Deserialize, Debug, Clone, Display, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
pub enum StatusZamowienia {
    ZamowieniePrzyjete,
    Wprzygotowaniu,
    OczekujeNaWysylke,
    Wpodrozy,
    Dostarczone,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AdminZamowieniaListView {
    id: i64,
    user_id: i64,
    date: String,
    #[serde(with = "rust_decimal::serde::str")]
    cena: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    vat: Decimal,
    numer_fv: String,
    oplacone: StatusOplacenia,
    status: StatusZamowienia,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AdminZamowieniaItemView {
    id: i64,
    email: String,
    tel: String,
    #[serde(flatten)]
    pub lokacja: ZamowienieLokacja,
    #[serde(flatten)]
    pub faktura_dane: Option<ZamowienieFV>,
    #[serde(flatten)]
    pub transport: Option<DaneTransportu>,
    pub imie: String,
    pub nazwisko: String,
}
#[derive(Serialize, Deserialize, FromRow, Debug, Clone)]
pub struct Zamowienie {
    pub id: i64,
    pub date: String,
    pub email: Option<String>,
    pub tel: Option<String>,
    #[serde(flatten)]
    pub lokacja: ZamowienieLokacja,

    #[serde(flatten)]
    pub faktura_dane: Option<ZamowienieFV>,
    #[serde(flatten)]
    pub transport: Option<DaneTransportu>,
    #[serde(with = "rust_decimal::serde::str")]
    // #[sqlx(try_from = "String")]
    pub vat: Decimal, //kwota vat
    pub numer_fv: String,
    pub oplacone: StatusOplacenia,
    pub status: StatusZamowienia,
    #[serde(with = "rust_decimal::serde::str")]
    // #[sqlx(try_from = "String")]
    pub cena: Decimal, //kwota netto
    pub user_id: Option<i64>,
    pub imie: String,
    pub nazwisko: String,
    pub waluta: Waluta,
}
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ZamowienieLokacja {
    pub ulica: String,
    pub miasto: String,
    pub kod_pocztowy: String,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DaneTransportu {
    pub odleglosc_km: f64,
    #[serde(with = "rust_decimal::serde::str")]
    // #[sqlx(try_from = "String")]
    pub cena_netto: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    // #[sqlx(try_from = "String")]
    pub stawka_vat: Decimal,
}
impl DaneTransportu {
    pub fn new(odleglosc_km: f64, cena_netto: Decimal, stawka_vat: Decimal) -> Self {
        Self{
            odleglosc_km,
            cena_netto,
            stawka_vat,
        }
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ZamowienieFV {
    #[serde(alias = "fv_ulica")]
    pub ulica: Option<String>,
    #[serde(alias = "fv_miasto")]
    pub miasto: Option<String>,
    #[serde(alias = "fv_kod_pocztowy")]
    pub kod_pocztowy: Option<String>,
    pub nip: String,
    pub nazwa_firmy: String,
}

#[derive(Serialize, Deserialize, FromRow, Debug, Clone)]
pub struct ZamowieniePozycja {
    #[serde(skip_deserializing)]
    pub zamowienie_id: i64, //id z Zamowienie
    pub product_id: i64,
    pub ilosc: i64,
    #[serde(with = "rust_decimal::serde::str")]
    // #[sqlx(try_from = "String")]
    pub cena: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    // #[sqlx(try_from = "String")]
    pub vat: Decimal,
    pub konfiguracja: serde_json::Value,
}
#[derive(sqlx::FromRow)]
struct LastOrderData {
    _date: String,
    numer_fv: String,
}
pub async fn generate_fv_number(pool: &SqlitePool) -> Result<String, sqlx::Error> {
    let now = Local::now();
    let current_year = now.year();
    let current_month = now.month();

    let last_order = get_last_order_number(pool).await?;

    let new_number = if let Some(order) = last_order {
        // order.numer_fv = "FV/MM/YYYY/NR"
        let parts: Vec<&str> = order.numer_fv.split('/').collect();
        let last_month: u32 = parts[1].parse().unwrap_or(0);
        let last_year: i32 = parts[2].parse().unwrap_or(0);
        let last_seq: u32 = parts[3].parse().unwrap_or(0);

        if last_month == current_month && last_year == current_year {
            last_seq + 1
        } else {
            1
        }
    } else {
        1
    };

    Ok(format!(
        "FV/{:02}/{}/{:03}",
        current_month, current_year, new_number
    ))
}
async fn get_last_order_number(pool: &SqlitePool) -> Result<Option<LastOrderData>, sqlx::Error> {
    sqlx::query_as::<_, LastOrderData>("SELECT date, numer_fv FROM orders ORDER BY id DESC LIMIT 1")
        .fetch_optional(pool)
        .await
}

impl Default for Zamowienie {
    fn default() -> Self {
        Self{
            id: 0,
            user_id: None,
            imie: String::new(),
            nazwisko: String::new(),
            date: chrono::Local::now()
                .format("%Y-%m-%d | %H:%M:%S")
                .to_string(),
            email: None,
            tel: None,
            lokacja: ZamowienieLokacja::default(),
            faktura_dane: None,
            transport: None,
            cena: Decimal::ZERO,
            vat: Decimal::ZERO,
            waluta: Waluta::Pln,
            numer_fv: String::new(),
            oplacone: StatusOplacenia::Nieoplacone,
            status: StatusZamowienia::ZamowieniePrzyjete,
        }
    }
}
impl Zamowienie {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn add_user_id(mut self, val: Option<i64>) -> Self {
        self.user_id = val;
        self
    }
    pub fn add_email(mut self, val: Option<impl Into <String>>) -> Self {
        self.email = val.map(|e| e.into());
        self
    }
    pub fn add_tel(mut self, val: Option<impl Into <String>>) -> Self {
        self.tel = val.map(|e| e.into());
        self
    }
    pub fn add_fv(mut self, val: Option<ZamowienieFV>) -> Self {
        self.faktura_dane = val;
        self
    }
    pub fn add_transport(mut self, val: Option<DaneTransportu>) -> Self {
        self.transport = val;
        self
    }
    pub fn add_imie(mut self, val: String) -> Self {
        self.imie = val;
        self
    }
    pub fn add_nazwisko(mut self, val: String) -> Self {
        self.nazwisko = val;
        self
    }
    pub fn add_cena(mut self, val: Decimal) -> Self {
        self.cena = val;
        self
    }
    pub fn add_vat(mut self, val: Decimal) -> Self {
        self.vat = val;
        self
    }
    pub async fn generuj_nr_fv(mut self, pool: &SqlitePool) -> Self {
        let numer = generate_fv_number(pool).await.ok().unwrap_or_default();
        self.numer_fv = numer;
        self
    }
    pub fn add_lokacja(mut self, val: ZamowienieLokacja) -> Self {
        self.lokacja = val;
        self
    }
}
