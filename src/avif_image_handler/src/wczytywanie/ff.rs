use image::{DynamicImage, ImageBuffer, Rgba};

/// # Decoding farbfeld
/// ;)
pub fn ff(bajty: &[u8]) -> Result<DynamicImage, std::io::Error> {
    // 1. Sprawdzenie nagłówka "farbfeld" (8 bajtów)
    if bajty.len() < 16 || &bajty[0..8] != b"farbfeld" {
        return Err(std::io::Error::other("To nie jest poprawny plik Farbfeld"));
    }

    // 2. Odczyt wymiarów (W i H to 32-bitowe wartości Big Endian)
    let width = u32::from_be_bytes([bajty[8], bajty[9], bajty[10], bajty[11]]);
    let height = u32::from_be_bytes([bajty[12], bajty[13], bajty[14], bajty[15]]);

    // 3. Obliczenie oczekiwanej długości danych
    // Każdy piksel to 4 kanały (RGBA) po 2 bajty = 8 bajtów na piksel
    let expected_len = 16 + (width as usize * height as usize * 8);
    if bajty.len() < expected_len {
        return Err(std::io::Error::other("Plik Farbfeld jest ucięty"));
    }

    // 4. Konwersja 16-bit BE na 8-bit (DynamicImage najlepiej radzi sobie z 8-bit RGBA w assetach)
    // Farbfeld używa liniowego 16-bit, ale większość silników i tak chce 8-bit.
    let mut rgba8_pixels = Vec::with_capacity((width * height * 4) as usize);

    // Dane pikseli zaczynają się od 16 bajtu
    let pixel_data = &bajty[16..expected_len];

    for chunk in pixel_data.chunks_exact(8) {
        // chunk to [R, R, G, G, B, B, A, A]
        // Pobieramy tylko starszy bajt (MSB), co jest najszybszą metodą konwersji 16->8 bit
        let r = chunk[0];
        let g = chunk[2];
        let b = chunk[4];
        let a = chunk[6];

        rgba8_pixels.push(r);
        rgba8_pixels.push(g);
        rgba8_pixels.push(b);
        rgba8_pixels.push(a);
    }

    let buf = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, rgba8_pixels)
        .ok_or_else(|| std::io::Error::other("Błąd tworzenia bufora RGBA8 z Farbfeld"))?;


    Ok(DynamicImage::ImageRgba8(buf))
}