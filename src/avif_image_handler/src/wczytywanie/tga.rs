use image::codecs::tga::TgaDecoder;
use image::{ColorType, DynamicImage, ImageBuffer, ImageDecoder};
use std::io::Cursor;
/// # Decoding tga
/// ;)
pub fn tga(bajty: &Vec<u8>) -> Result<DynamicImage, std::io::Error> {
    let cursor = Cursor::new(bajty);
    let decoder = TgaDecoder::new(cursor)
        .map_err(std::io::Error::other)?;

    let (width, height) = decoder.dimensions();
    let color_type = decoder.color_type();

    let mut pixels = vec![0u8; decoder.total_bytes() as usize];
    decoder.read_image(&mut pixels)
        .map_err(std::io::Error::other)?;

    let obraz = match color_type {
        ColorType::L8 => {
            let buf = ImageBuffer::<image::Luma<u8>, _>::from_raw(width, height, pixels)
                .ok_or_else(|| std::io::Error::other("Błąd bufora TGA L8"))?;
            DynamicImage::ImageLuma8(buf)
        },
        ColorType::Rgb8 => {
            // TgaDecoder w 'image' zazwyczaj już zamienił BGR -> RGB dla Rgb8.
            // Ale jeśli obrazy wychodzą niebieskie, tutaj należałoby zrobić swap.
            // for chunk in pixels.chunks_exact_mut(3) {
            //     chunk.swap(0, 2); // Zamienia miejscami R i B
            // }
            let buf = ImageBuffer::<image::Rgb<u8>, _>::from_raw(width, height, pixels)
                .ok_or_else(|| std::io::Error::other("Błąd bufora TGA RGB8"))?;
            DynamicImage::ImageRgb8(buf)
        },
        ColorType::Rgba8 => {

            // Ręczny swap BGRA -> RGBA
            // for chunk in pixels.chunks_exact_mut(4) {
            //     chunk.swap(0, 2); // Zamienia miejscami R i B, zostawia A na miejscu
            // }
            //
            let buf = ImageBuffer::<image::Rgba<u8>, _>::from_raw(width, height, pixels)
                .ok_or_else(|| std::io::Error::other("Błąd bufora TGA RGBA8"))?;
            DynamicImage::ImageRgba8(buf)
        },
        // Obsługa 16-bit (5-5-5-1)
        _ if pixels.len() == (width * height * 2) as usize => {
            let mut rgb8_pixels = Vec::with_capacity((width * height * 3) as usize);
            for chunk in pixels.chunks_exact(2) {
                let pixel = u16::from_le_bytes([chunk[0], chunk[1]]);

                // TGA 16-bit: xRRRRRGG GGGBBBBB
                let r = (((pixel >> 10) & 0x1F) as u8) << 3;
                let g = (((pixel >> 5) & 0x1F) as u8) << 3;
                let b = ((pixel & 0x1F) as u8) << 3;

                rgb8_pixels.push(r);
                rgb8_pixels.push(g);
                rgb8_pixels.push(b);
            }
            let buf = ImageBuffer::<image::Rgb<u8>, _>::from_raw(width, height, rgb8_pixels)
                .ok_or_else(|| std::io::Error::other("Błąd konwersji TGA 16->RGB8"))?;
            DynamicImage::ImageRgb8(buf)
        },
        _ => return Err(std::io::Error::new(std::io::ErrorKind::Unsupported, format!("Nieobsługiwany ColorType TGA: {:?}", color_type))),
    };

    Ok(obraz)
}