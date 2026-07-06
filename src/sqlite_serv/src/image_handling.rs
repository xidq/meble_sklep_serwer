use crate::sql::{get_id_and_name_id, insert_product_image, AppState, Rozdzielczosci};
use avif_image_handler::save::avif_match;
use avif_image_handler::wczytywanie::main_wczytywanie::wczytaj_pliki;
use axum::extract::Multipart;
use axum::extract::State;
use axum::response::IntoResponse;
use sqlx::sqlite::SqlitePool;
use std::fs::{create_dir, read_dir, remove_file};
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;

pub fn image_folders(name: String) -> (PathBuf, String){
    let nazwa_folderu: String = name.split('_')
        .take(2)
        .collect::<Vec<&str>>()
        .join("_");
    let dir = PathBuf::from(format!("src/api/images/{}",nazwa_folderu));
    if !dir.exists(){
        create_dir(&dir).ok();
    }
    (dir, nazwa_folderu )
}

pub async fn image_handle() -> Result<Vec<String>, std::io::Error>{

    let path = "src/api/images/queued";

    let mut ghhh: Vec<String> = Vec::new();

    if let Ok(entries) = read_dir(path) {

        let file_paths: Vec<PathBuf> = entries
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().is_file())
            .map(|entry| entry.path())
            .collect();

        // produkt_versja_wariant.rozszerzenie
        for path in &file_paths {

            let (foto, nazwa_org) = match wczytaj_pliki(path.clone()) {
                Ok(result) => result,
                Err(e) => {
                    println!("error wczytywania: {}", e);
                    continue;
                }
            };

            let (folder, ghhhh) = image_folders(nazwa_org.clone());

            match avif_match(nazwa_org, foto, &folder).await {
                Ok(()) => (),
                Err(e) => println!("error: {}", e),
            };

            ghhh.push(ghhhh);

            remove_file(path).ok();

        }
    }

    Ok(ghhh)

}

// pub async fn image_upload_to_server_handle(
//     State(state): State<AppState>,
//     mut multipart: Multipart,
// ) -> impl IntoResponse {
//     let base_path = "src/api/images/queued";
//
//     // Upewnij się, że folder istnieje
//     if let Err(e) = create_dir_all(base_path).await {
//         return (http::StatusCode::INTERNAL_SERVER_ERROR, format!("Błąd folderu: {}", e)).into_response();
//     }
//
//     // 1. Odbieranie i zapisywanie plików
//     while let Ok(Some(field)) = multipart.next_field().await {
//         let file_name = field.file_name().unwrap_or("unknown").to_string();
//         let path = Path::new(base_path).join(&file_name);
//
//         if let Ok(data) = field.bytes().await {
//             if let Ok(mut file) = File::create(&path).await {
//                 let _ = file.write_all(&data).await;
//             }
//         }
//     }
//
//     // 2. Wywołanie logiki przetwarzania (z bazy danych)
//     // Używamy tokio::spawn, jeśli chcesz, aby klient dostał odpowiedź szybciej,
//     // a przetwarzanie trwało w tle. Jeśli musisz czekać na koniec:
//     match image_database_compare_and_sht(&state.db).await {
//         Ok(_) => (http::StatusCode::OK, "Pliki przetworzone").into_response(),
//         Err(e) => (http::StatusCode::INTERNAL_SERVER_ERROR, format!("Błąd: {}", e)).into_response(),
//     }
//
//     // Zamiast czekać na porównywanie/przetwarzanie w bazie,
//     // tylko wysyłamy sygnał "hej, coś przyszło, ogarnij to!"
//     let _ = state.tx.send(()).await;
//
//     // Szybka odpowiedź do klienta
//     (http::StatusCode::ACCEPTED, "Zadanie przyjęte do kolejki").into_response()
// }
pub async fn image_upload_to_server_handle(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let base_path = "src/api/images/queued";

    // Zapisywanie plików (I/O)
    while let Ok(Some(field)) = multipart.next_field().await {
        let file_name = field.file_name().unwrap_or("unknown").to_string();
        let path = std::path::Path::new(base_path).join(&file_name);

        if let Ok(data) = field.bytes().await &&
            let Ok(mut file) = tokio::fs::File::create(&path).await {
                let _ = file.write_all(&data).await;

        }
    }

    // Informujemy workera, że pojawiły się nowe pliki do obróbki
    let _ = state.tx.send(()).await;

    (http::StatusCode::ACCEPTED, "Pliki zapisane, przetwarzanie w tle").into_response()
}

pub async fn image_database_compare_and_sht(
    pool: &SqlitePool,
) -> Result<(), sqlx::Error>{
    //pobiera id oraz name_id z bazy
    let id_vec = get_id_and_name_id(pool).await?;

    let photo_vec = image_handle().await?;

    for photo_name in photo_vec {
        let name_id = photo_name.split('_').take(2).collect::<Vec<&str>>().join("_");

        // 1. Ustalenie ID produktu
        let product_id = if let Some(&(id, _)) = id_vec.iter().find(|(_, name)| name == &name_id) {
            id // Produkt istnieje
        } else {
            // Produktu nie ma, tworzymy nowy (zwraca nowe ID)
            generate_new_id_from_db(pool, &name_id).await?
        };

        // 2. Skanowanie folderu z obrazami dla danego produktu
        // Ścieżka: src/api/images/{photo_name}
        let folder_path = format!("src/api/images/{}", photo_name);
        if let Ok(entries) = std::fs::read_dir(folder_path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_file() {
                    // Parsowanie nazwy: produkt_ver_wariant_rozdzielczosc
                    // Zakładając, że plik to: nazwa_ver_y_z.jpg
                    let file_name = path.file_name().unwrap().to_str().unwrap();
                    let parts: Vec<&str> = file_name.split('_').collect();

                    if parts.len() >= 4 {
                        // Z - rozdzielczość (ostatnia część przed rozszerzeniem)
                        let res_str = parts[3].split('.').next().unwrap_or("0");
                        let res_val: i32 = res_str.parse().unwrap_or(0);
                        //
                        // // Konwersja na enum
                        // if let Ok(res) = res_val {
                        //     // 3. Aktualizacja/Wstawianie do bazy
                        //     // Tutaj logika: Sprawdź czy istnieje w product_images, jak tak to UPDATE path, jak nie to INSERT
                        //     insert_product_image(pool, product_id, res, path.to_str().unwrap()).await?;
                        // }
                        match res_val {
                            16 | 32 | 64 | 128 | 256 | 512 | 1024 | 2048 => {
                                // Skoro jest to i32 i wiemy, że pasuje do enuma, rzutujemy:
                                let res: Rozdzielczosci = unsafe { std::mem::transmute(res_val) };

                                // 3. Aktualizacja/Wstawianie do bazy
                                insert_product_image(pool, product_id, res, path.to_str().unwrap()).await?;
                            },
                            _ => {
                                // Logujemy błąd lub pomijamy plik
                                println!("Nieobsługiwana rozdzielczość: {}", res_val);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())

}


pub async fn generate_new_id_from_db(
    pool: &SqlitePool,
    _name_id: &str,
) -> Result<i64, sqlx::Error> {
    let prefix: i64 = 1;
    let base_multiplier = 1_000_000_000;
    let base_id = prefix * base_multiplier;

    // Używamy query_scalar, bo pobieramy tylko jedną kolumnę (MAX(id))
    // query_scalar automatycznie wyciąga wartość z pierwszego pola
    let max_id: Option<i64> = sqlx::query_scalar(
        "SELECT MAX(id) FROM products WHERE id >= ? AND id < ?"
    )
        .bind(base_id)
        .bind(base_id + base_multiplier)
        .fetch_one(pool)
        .await?; // Używamy ? do wyciągnięcia Result z fetch_one

    // Obliczenie nowego ID
    let next_id = match max_id {
        Some(id) => id + 1,
        None => base_id + 1,
    };

    Ok(next_id)
}