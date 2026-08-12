pub mod get;

use anyhow::anyhow;
use serde::Deserialize;
use reqwest::header::USER_AGENT;
use rust_decimal::{dec, Decimal};
use crate::zamowienia::DaneTransportu;

// Współrzędne Twojego sklepu/magazynu (np. Rybnik, Polska)
// Podmień te wartości na współrzędne swojego rzeczywistego punktu startowego!
const MAGAZYN_LON: f64 = 18.4444889;
const MAGAZYN_LAT: f64 = 50.0908261;

// Struktura pomocnicza do odebrania współrzędnych z Nominatim
#[derive(Deserialize, Debug)]
struct GeocodeResult {
    lat: String,
    lon: String,
}

#[derive(Deserialize, Debug)]
struct OsrmResponse {
    routes: Vec<Route>,
}

#[derive(Deserialize, Debug)]
struct Route {
    distance: f64, // Dystans w metrach
}

pub async fn oblicz_odleglosc_do_klienta(
    ulica: &str,
    miasto: &str,
    kod_pocztowy: &str,
) -> anyhow::Result<DaneTransportu> {
    let client = reqwest::Client::new();

    // ==========================================
    // KROK 1: Geokodowanie (Adres -> Współrzędne)
    // ==========================================

    // Ważne: Nominatim WYMAGA podania unikalnego User-Agent (inaczej zablokuje zapytanie)
    let user_agent_value = "Testserver_get_mock_adress_pos/1.0 (p.jerzu@gmail.com)";
    let geocode_url = "https://nominatim.openstreetmap.org/search";

    // Bezpieczne, ustrukturyzowane parametry zapytania
    let params = [
        ("street", ulica),
        ("city", miasto),
        ("postalcode", kod_pocztowy),
        ("country", "Poland"),
        ("format", "json"),
        ("limit", "1"),
    ];

    let geocode_response: Vec<GeocodeResult> = client
        .get(geocode_url)
        .header(USER_AGENT, user_agent_value)
        .query(&params)
        .send()
        .await?
        .json()
        .await?;
    let najlepsze_dopasowanie = geocode_response.first().ok_or_else(|| {
        anyhow!("Nie udało się odnaleźć adresu: ul. {}, {} {}", ulica, kod_pocztowy, miasto)
    })?;

    let dest_lat: f64 = najlepsze_dopasowanie.lat.parse()?;
    let dest_lon: f64 = najlepsze_dopasowanie.lon.parse()?;



    // ==========================================
    // KROK 2: Routing (Współrzędne -> Kilometry)
    // ==========================================

    // Zapytanie do OSRM w formacie: driving/lon_start,lat_start;lon_koniec,lat_koniec
    let osrm_url = format!(
        "http://router.project-osrm.org/route/v1/driving/{},{};{},{}?overview=false",
        MAGAZYN_LON, MAGAZYN_LAT, dest_lon, dest_lat
    );

    let osrm_response: OsrmResponse = client
        .get(&osrm_url)
        .header(USER_AGENT, user_agent_value)
        .send()
        .await?
        .json()
        .await?;

    let route = osrm_response.routes.first()
        .ok_or_else(|| anyhow!("Nie udało się wyznaczyć trasy drogowej do podanego adresu"))?;

    // Przeliczamy metry na kilometry i zaokrąglamy do 1 miejsca po przecinku (np. 12.4 km)
    let odleglosc_km = Decimal::new(route.distance as i64, 4);
    // let odleglosc_km_zaokr = (odleglosc_km * 10.0).round() / 10.0;

    let spalanie_na_1km = Decimal::new(125, 3);
    let paliwo_cena = Decimal::new(699, 2);
    let margines = Decimal::new(11, 1); // %
    let sum_km = odleglosc_km * dec!(2.0);
    let wynagrodzenie_za_km = Decimal::new(7, 0);
    let paliwo = paliwo_cena * spalanie_na_1km;
    let dodatek = wynagrodzenie_za_km * sum_km;
    let kwota_za_trase = (paliwo * sum_km) * margines + dodatek;
    let out = DaneTransportu::new(f64::try_from(odleglosc_km)?, kwota_za_trase, Decimal::new(23, 2));
    Ok(out)

}