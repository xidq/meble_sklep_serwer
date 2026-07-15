use image::codecs::qoi::QoiDecoder;
use std::io::Cursor;
use image::{ColorType, DynamicImage, ImageBuffer, ImageDecoder};
/// # Decoding qoi
/// ;)
pub fn qoi(bajty: &Vec<u8>) -> Result<DynamicImage, std::io::Error> {
    let cursor = Cursor::new(bajty);

    // Inicjalizacja dekodera QOI
    let decoder = QoiDecoder::new(cursor)
        .map_err(std::io::Error::other)?;

    let (width, height) = decoder.dimensions();
    let color_type = decoder.color_type();

    // Alokujemy bufor na surowe dane pikseli
    let mut pixels = vec![0u8; decoder.total_bytes() as usize];
    decoder.read_image(&mut pixels)
        .map_err(std::io::Error::other)?;

    // Mapowanie na DynamicImage
    let obraz = match color_type {
        ColorType::Rgb8 => {
            let buf = ImageBuffer::<image::Rgb<u8>, _>::from_raw(width, height, pixels)
                .ok_or_else(|| std::io::Error::other("Błąd bufora QOI RGB8"))?;
            DynamicImage::ImageRgb8(buf)
        },
        ColorType::Rgba8 => {
            let buf = ImageBuffer::<image::Rgba<u8>, _>::from_raw(width, height, pixels)
                .ok_or_else(|| std::io::Error::other("Błąd bufora QOI RGBA8"))?;
            DynamicImage::ImageRgba8(buf)
        },
        _ => return Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            format!("Format QOI obsługuje tylko RGB8 i RGBA8, otrzymano: {:?}", color_type)
        )),
    };

    // QOI nie posiada metadanych ICC ani EXIF w nagłówku ani w stopce pliku.
    Ok(obraz)
}