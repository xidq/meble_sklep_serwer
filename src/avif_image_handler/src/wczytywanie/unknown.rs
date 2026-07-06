use std::io::Cursor;
use image::{DynamicImage, ImageReader};
/// # Decoding unknown
/// Trying to guess what's the format.
/// 
/// Here ImageReader is used.
pub fn unknown(bajty: &[u8]) -> Result<DynamicImage, std::io::Error> {
    let cursor = Cursor::new(bajty);

    // Próba zgadnięcia formatu na podstawie bajtów (magiczne liczby)
    let reader = ImageReader::new(cursor)
        .with_guessed_format()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    // Jeśli format nie został rozpoznany, wywalamy błąd
    if reader.format().is_none() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Nie rozpoznano formatu pliku (brak znanej sygnatury)"
        ));
    }

    // Dekodowanie do DynamicImage
    let obraz = reader.decode()
        .map_err(std::io::Error::other)?;

    // Dla nieznanych formatów trudno o generyczne wyciąganie ICC/EXIF bez matchowania,
    // więc zwracamy None.
    Ok(obraz)
}