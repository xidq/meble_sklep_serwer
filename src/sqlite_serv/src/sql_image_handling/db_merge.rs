use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::string::ToString;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use anyhow::Result;
use serde::de::Unexpected::Str;
use id_handling::enums_structs::Product;
use crate::sql::{get_id_and_name_id, get_images_according_to_id_name, get_product_data_by_id};

// #[derive(Serialize, Deserialize, Clone, Debug)]
// pub struct DbMerge{
//     dane_produktu: Product,
//     dane_zdjec: ZdjeciaNaSerwerNorm,
// }
type Rozdzielczosc = String; // np. "16", "32"
type Wariant = String;      // np. "var_1", "var_2"

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DbMerge {
    pub dane_produktu: Product,

    // Spłaszczamy mapę w strukturze za pomocą adnotacji serde(flatten).
    // Dzięki temu w JSON-ie klucze "var_1", "var_2" pojawią się na tym samym poziomie co "dane_produktu"
    #[serde(flatten)]
    pub warianty_zdjec: BTreeMap<Wariant, BTreeMap<Rozdzielczosc, String>>,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum LokalizacjaNaFrontendzie{
    Images,
    Data,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ZdjeciaNaSerwerNorm{
    r16: String,
    r32: String,
    r64: String,
    r128: String,
    r256: String,
    r512: String,
    r1024: String,
    r2048: String,
}
impl Default for ZdjeciaNaSerwerNorm{
    fn default() -> Self {

        ZdjeciaNaSerwerNorm{
            r16: String::new(),
            r32: String::new(),
            r64: String::new(),
            r128: String::new(),
            r256: String::new(),
            r512: String::new(),
            r1024: String::new(),
            r2048: String::new(),
        }
    }
}
impl LokalizacjaNaFrontendzie {
    pub fn get_localisation(&self) -> String{
        match self {
            Self::Images => "data/img_front/".to_string(),
            Self::Data => "data/".to_string(),
        }
    }
}
pub async fn ogarnianie_porownania_przedmiotu_i_zdjec_zbieranie_danych(
    id: i64,
    pool_products: &SqlitePool, // Baza produktów (my_database.db)
    pool_images: &SqlitePool,   // Baza zdjęć (images.db)
) -> Result<(DbMerge, Vec<String>), sqlx::Error>{

    let lista_ids_i_nazw_id = get_id_and_name_id(pool_products).await?;


    let name_id = lista_ids_i_nazw_id
        .iter()
        .find(|item| item.0 == id)
        .map(|item| item.1.clone())
        .ok_or(sqlx::Error::RowNotFound)?;

    let foty = get_images_according_to_id_name(name_id, pool_images).await?;
    let foty_serv = zmiany_sciezek_fot_pod_serwer(foty.clone());
    let foty_norm = normalizacja_sciezek_wariantow_na_json(foty_serv);
    let dane = get_product_data_by_id(id, pool_products).await?;

    let out = DbMerge{
        dane_produktu: dane,
        warianty_zdjec: foty_norm,
    };

    Ok((out, foty))

}
pub fn zmiany_sciezek_fot_pod_serwer(dane: Vec<String>) -> Vec<String> {
    let sciezka_bazowa = "../data/img_front/";
    let znacznik = "images/";

    dane.iter()
        .map(|path| {
            // Szukamy, gdzie w tekście zaczyna się "images/"
            if let Some(pos) = path.find(znacznik) {
                // Odcinamy wszystko do końca słowa "images/"
                // pos + znacznik.len() daje nam pozycję zaraz ZA "images/"
                let reszta_sciezki = &path[pos + znacznik.len()..];

                // Łączymy: "../img/img_front/" + "test_1/test_1_1_256.avif"
                format!("{}{}", sciezka_bazowa, reszta_sciezki)
            } else {
                // Jeśli z jakiegoś powodu w ścieżce nie ma "images/",
                // zwracamy bezpiecznie oryginalną ścieżkę
                path.clone()
            }
        })
        .collect()
}
pub fn normalizacja_sciezek_wariantow_na_json(sciezki: Vec<String>) -> BTreeMap<String, BTreeMap<String, String>> {
    let mut warianty = BTreeMap::new();

    for path in sciezki {
        // 1. Pobieramy tylko nazwę pliku, szukając ostatniego ukośnika '/'
        let nazwa_pliku = path.split('/').last().unwrap_or(&path);

        // 2. Odcinamy rozszerzenie .avif (bierzemy wszystko przed kropką)
        if let Some(czysty_tekst) = nazwa_pliku.split('.').next() {

            // 3. Tniemy po '_' -> ["test", "1", "3", "16"]
            let czesci: Vec<&str> = czysty_tekst.split('_').collect();

            if czesci.len() >= 2 {
                // Dwa ostatnie elementy to rozdzielczość i wariant
                let rozdzielczosc = czesci[czesci.len() - 1].to_string(); // "16"
                let numer_wariantu = czesci[czesci.len() - 2];            // "3"
                let klucz_wariantu = format!("var_{}", numer_wariantu);    // "var_3"

                warianty
                    .entry(klucz_wariantu)
                    .or_insert_with(BTreeMap::new)
                    .insert(rozdzielczosc, path);
            }
        }
    }

    warianty
}


pub async fn send_json_to_frontend<T>(json_data: &T, target_path: LokalizacjaNaFrontendzie, modyfikator: String, nazwa: String) -> Result<()>
where
    T: Serialize,
{
    // 1. Konwertujemy strukturę Rusta na ładnie sformatowany tekst JSON (z wcięciami)
    let json_string = serde_json::to_string_pretty(json_data)?;

    let frontend_server = std::env::var("FRONTEND_SERVER")
        .unwrap_or_else(|_| "https://localhost:8444/".to_string());

    let serwer_adres = frontend_server.to_owned() + &target_path.get_localisation() + &modyfikator + "/" + &nazwa + ".json";
    // 3. Zamiast File::create -> wysyłamy to bezpośrednio do serwera POST-em
    reqwest::Client::new()
        .post(&serwer_adres)
        .header("Content-Type", "application/json")
        .body(json_string)
        .send()
        .await?;

    println!("Sukces! Dane zostały zrzucone do pliku dla frontendu.");
    Ok(())
}

pub async fn przeslij_plik_graficzny_na_serwer(
    lokalna_sciezka_pliku: Vec<String>,
    target_path: LokalizacjaNaFrontendzie,
    modyfikator: String
) -> Result<()> {

    for xx in lokalna_sciezka_pliku {
        // 1. Bezpiecznie otwieramy i wczytujemy plik do bufora bajtów
        let mut plik = File::open(Path::new(&xx))?;
        let mut bufor = Vec::new();
        plik.read_to_end(&mut bufor)?;

        // 2. Dynamicznie wyciągamy fizyczną nazwę pliku ze ścieżki (np. "test_1_1_16.avif")
        // Używamy bezpiecznego splitu po '/', żeby działało na każdym systemie
        let czysta_nazwa_pliku = xx.split('/').last().unwrap_or(&xx);
        let frontend_server = std::env::var("FRONTEND_SERVER")
            .unwrap_or_else(|_| "https://localhost:8444/".to_string());
        // 3. Budujemy poprawny adres URL (celujemy w serwer_dev/img/img_front/test_1/test_1_1_16.avif)
        let serwer_adres = format!(
            "{}{}{}/{}",
            frontend_server,
            target_path.get_localisation(),
            modyfikator,
            czysta_nazwa_pliku
        );

        // 4. Wysyłamy żądanie POST z bajtami w body
        reqwest::Client::new()
            .post(&serwer_adres)
            .header("Content-Type", "application/octet-stream")
            .body(bufor)
            .send()
            .await?;

        println!("[Rust] Pomyślnie wysłano plik graficzny: {}", czysta_nazwa_pliku);
    }

    Ok(())
}
