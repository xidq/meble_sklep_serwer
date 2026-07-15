use png::{Decoder, ColorType, BitDepth};
use std::io::Cursor;
use image::{DynamicImage, ImageBuffer};
/// # Decoding png
/// ;)
pub fn png(bajty: &Vec<u8>) -> Result<DynamicImage, std::io::Error> {
    let cursor = Cursor::new(bajty);
    let decoder = Decoder::new(cursor);


    // decoder.set_transformations(png::Transformations::EXPAND);

    // Ważne: musimy poinstruować dekoder, aby czytał metadane, 
    // bo domyślnie może je zignorować dla oszczędności pamięci.
    let mut reader = decoder.read_info()
        .map_err(std::io::Error::other)?;

    let mut pixels = vec![0; reader.output_buffer_size().expect("nie ma pikseli w png")];
    let info = reader.next_frame(&mut pixels)
        .map_err(std::io::Error::other)?;
    

    // --- 3. Mapowanie na DynamicImage ---
    let width = info.width;
    let height = info.height;

    let obraz = match (info.color_type, info.bit_depth) {
        (ColorType::Rgb, BitDepth::Eight) => {
            let buf = ImageBuffer::<image::Rgb<u8>, _>::from_raw(width, height, pixels)
                .ok_or_else(|| std::io::Error::other("Błąd bufora PNG RGB8"))?;
            DynamicImage::ImageRgb8(buf)
        },
        (ColorType::Indexed, BitDepth::Eight) => {
            let bpp = pixels.len() / (width as usize * height as usize);

            match bpp {
                // Jeśli po EXPAND mamy 1 bajt na piksel, to znaczy, że paleta była
                // de facto skalą szarości i biblioteka to zoptymalizowała.
                1 => {
                    let buf = ImageBuffer::<image::Luma<u8>, _>::from_raw(width, height, pixels)
                        .ok_or_else(|| std::io::Error::other("Błąd bufora PNG L8 z palety"))?;
                    DynamicImage::ImageLuma8(buf)
                },
                // Jeśli mamy 3 bajty, a wiemy, że to skan, możemy albo zostawić RGB,
                // albo ręcznie zdesaturować do Luma8, żeby oszczędzić ram.
                3 => {
                    let buf = ImageBuffer::<image::Rgb<u8>, _>::from_raw(width, height, pixels)
                        .ok_or_else(|| std::io::Error::other("Błąd bufora PNG RGB8 z palety"))?;

                    // Opcjonalnie: jeśli chcesz wymusić Luma8 nawet gdy paleta ma kolory:
                    // DynamicImage::ImageLuma8(DynamicImage::ImageRgb8(buf).into_luma8())
                    DynamicImage::ImageRgb8(buf)
                },
                4 => {
                    let buf = ImageBuffer::<image::Rgba<u8>, _>::from_raw(width, height, pixels)
                        .ok_or_else(|| std::io::Error::other("Błąd bufora PNG RGBA8 z palety"))?;
                    DynamicImage::ImageRgba8(buf)
                },
                _ => return Err(std::io::Error::other(format!("Nietypowy bpp ({}) dla Indexed", bpp))),
            }
        },
        (ColorType::Rgba, BitDepth::Eight) => {
            let buf = ImageBuffer::<image::Rgba<u8>, _>::from_raw(width, height, pixels)
                .ok_or_else(|| std::io::Error::other("Błąd bufora PNG RGBA8"))?;
            DynamicImage::ImageRgba8(buf)
        },
        (ColorType::Rgb, BitDepth::Sixteen) => {
            let data_u16 = bytes_to_u16(&pixels);
            let buf = ImageBuffer::<image::Rgb<u16>, _>::from_raw(width, height, data_u16)
                .ok_or_else(|| std::io::Error::other("Błąd bufora PNG RGB16"))?;
            DynamicImage::ImageRgb16(buf)
        },
        (ColorType::Rgba, BitDepth::Sixteen) => {
            let data_u16 = bytes_to_u16(&pixels);
            let buf = ImageBuffer::<image::Rgba<u16>, _>::from_raw(width, height, data_u16)
                .ok_or_else(|| std::io::Error::other("Błąd bufora PNG RGBA16"))?;
            DynamicImage::ImageRgba16(buf)
        },
        (ColorType::Grayscale, BitDepth::Eight) => {
            let buf = ImageBuffer::<image::Luma<u8>, _>::from_raw(width, height, pixels)
                .ok_or_else(|| std::io::Error::other("Błąd bufora PNG L8"))?;
            DynamicImage::ImageLuma8(buf)
        },
        (ColorType::GrayscaleAlpha, BitDepth::Eight) => {
            let buf = ImageBuffer::<image::LumaA<u8>, _>::from_raw(width, height, pixels)
                .ok_or_else(|| std::io::Error::other("Błąd bufora PNG L8"))?;
            DynamicImage::ImageLumaA8(buf)
        },
        (ColorType::Grayscale, BitDepth::Sixteen) => {
            let data_u16 = bytes_to_u16(&pixels);
            let buf = ImageBuffer::<image::Luma<u16>, _>::from_raw(width, height, data_u16)
                .ok_or_else(|| std::io::Error::other("Błąd bufora PNG L16"))?;
            DynamicImage::ImageLuma16(buf)
        },
        (ColorType::GrayscaleAlpha, BitDepth::Sixteen) => {
            let data_u16 = bytes_to_u16(&pixels);
            let buf = ImageBuffer::<image::LumaA<u16>, _>::from_raw(width, height, data_u16)
                .ok_or_else(|| std::io::Error::other("Błąd bufora PNG L16"))?;
            DynamicImage::ImageLumaA16(buf)
        },
        _ => return Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "Format PNG nieobsługiwany")),
    };

    Ok(obraz)
}

// Funkcja pomocnicza do konwersji bajtów na u16 (Big Endian - standard PNG)
fn bytes_to_u16(data: &[u8]) -> Vec<u16> {
    data.chunks_exact(2)
        .map(|c| u16::from_be_bytes([c[0], c[1]]))
        .collect()
}