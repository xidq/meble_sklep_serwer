pub mod post;
pub mod get;
// mod ksef;

use chrono::{Datelike, Local};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CaloscioweZamowienie{
    dane: Zamowienie,
    przedmioty: Vec<ZamowieniePozycja>,
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
    pub vat: f32, //kwota vat
    pub numer_fv: String,
    pub oplacone: bool,
    pub cena: f32, //kwota netto
    pub user_id: Option<i64>,
    pub imie: String,
    pub nazwisko: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ZamowienieLokacja {
    pub ulica: String,
    pub miasto: String,
    pub kod_pocztowy: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DaneTransportu {
    pub odleglosc_km: f32,
    pub cena_netto: f32,
    pub stawka_vat: f32,
}
impl DaneTransportu {
    pub fn new(
        odleglosc_km: f32,
        cena_netto: f32,
        stawka_vat: f32,
    ) -> Self {
        Self{
            odleglosc_km,
            cena_netto,
            stawka_vat,
        }
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ZamowienieFV{
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
pub struct ZamowieniePozycja{
    #[serde(skip_deserializing)]
    pub zamowienie_id: i64, //id z Zamowienie
    pub product_id: i64,
    pub ilosc: i64,
    pub cena: f32,
    pub vat: f32,
    pub konfiguracja: serde_json::Value,
}
#[derive(sqlx::FromRow)]
struct LastOrderData{
    _date: String,
    number: String,
}
pub async fn generate_fv_number(pool: &SqlitePool) -> Result<String, sqlx::Error> {
    let now = Local::now();
    let current_year = now.year();
    let current_month = now.month();

    let last_order = get_last_order_number(pool).await?;

    let new_number = if let Some(order) = last_order {
        // order.numer_fv = "FV/MM/YYYY/NR"
        let parts: Vec<&str> = order.number.split('/').collect();
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

    Ok(format!("FV/{:02}/{}/{:03}", current_month, current_year, new_number))
}
async fn get_last_order_number(pool: &SqlitePool) -> Result<Option<LastOrderData>, sqlx::Error> {
    sqlx::query_as::<_, LastOrderData>(
        "SELECT date, numer_fv FROM orders ORDER BY id DESC LIMIT 1"
    )
        .fetch_optional(pool)
        .await
}
impl Zamowienie {
    pub async fn new(
        user_id: Option<i64>,
        email: Option<impl Into <String>>,
        tel: Option<impl Into <String>>,
        lokacja: ZamowienieLokacja,
        faktura_dane: Option<ZamowienieFV>,
        transport: Option<DaneTransportu>,
        imie: String,
        nazwisko: String,
        cena: f32,
        vat: f32,
        pool: &SqlitePool,
    ) -> Self{

        Self{
            id: 0,
            user_id,
            imie,
            nazwisko,
            date: chrono::Local::now().format("%Y-%m-%d | %H:%M:%S").to_string(),
            email: email.map(|e| e.into()),
            tel: tel.map(|t| t.into()),
            lokacja,
            faktura_dane,
            transport,
            cena, // kwota netto
            vat,  // kwota vat
            numer_fv: generate_fv_number(pool).await.ok().unwrap_or_default(),
            oplacone: false,
        }
    }
}
