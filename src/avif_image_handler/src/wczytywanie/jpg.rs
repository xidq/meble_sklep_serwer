use image::{DynamicImage, ImageBuffer};
use jpeg_decoder::Decoder;
use std::io::Cursor;

/// # Decoding jpg
pub fn jpeg(bajty: &Vec<u8>) -> Result<DynamicImage, std::io::Error> {

    let mut decoder = Decoder::new(Cursor::new(bajty));
    let pixels = decoder.decode()
        .map_err(std::io::Error::other)?;

    let info = decoder.info()
        .ok_or_else(|| std::io::Error::other( "Brak info o pliku"))?;


    let obraz = match info.pixel_format {
        jpeg_decoder::PixelFormat::L8 => {
            let buf = ImageBuffer::<image::Luma<u8>, _>::from_raw(info.width as u32, info.height as u32, pixels)
                .ok_or_else(|| std::io::Error::other("Błąd bufora L8"))?;
            DynamicImage::ImageLuma8(buf)
        },
        jpeg_decoder::PixelFormat::L16 => {
            // jpeg-decoder zwraca Vec<u8>, więc dla 16-bitów musimy złączyć bajty
            let data_u16: Vec<u16> = pixels.chunks_exact(2)
                .map(|c| u16::from_be_bytes([c[0], c[1]]))
                .collect();
            let buf = ImageBuffer::<image::Luma<u16>, _>::from_raw(info.width as u32, info.height as u32, data_u16)
                .ok_or_else(|| std::io::Error::other("Błąd bufora L16"))?;
            DynamicImage::ImageLuma16(buf)
        },
        jpeg_decoder::PixelFormat::RGB24 => {
            let buf = ImageBuffer::<image::Rgb<u8>, _>::from_raw(info.width as u32, info.height as u32, pixels)
                .ok_or_else(|| std::io::Error::other("Błąd bufora RGB24"))?;
            DynamicImage::ImageRgb8(buf)
        },
        jpeg_decoder::PixelFormat::CMYK32 => {
            let mut rgb_pixels = Vec::with_capacity((pixels.len() / 4) * 3);

            for cmyk in pixels.chunks_exact(4) {
                // W JPEG bajty CMYK są często składowymi 0..255
                let c = cmyk[0] as f32 / 255.0;
                let m = cmyk[1] as f32 / 255.0;
                let y = cmyk[2] as f32 / 255.0;
                let k = cmyk[3] as f32 / 255.0;

                // Standardowy wzór matematyczny:
                // R = 255 * (1-C) * (1-K)
                // G = 255 * (1-M) * (1-K)
                // B = 255 * (1-Y) * (1-K)

                let r = 255.0 * (1.0 - c) * (1.0 - k);
                let g = 255.0 * (1.0 - m) * (1.0 - k);
                let b = 255.0 * (1.0 - y) * (1.0 - k);

                rgb_pixels.push(r.round() as u8);
                rgb_pixels.push(g.round() as u8);
                rgb_pixels.push(b.round() as u8);
            }

            let buf = ImageBuffer::<image::Rgb<u8>, _>::from_raw(info.width as u32, info.height as u32, rgb_pixels)
                .ok_or_else(|| std::io::Error::other("Błąd bufora RGB z CMYK"))?;
            DynamicImage::ImageRgb8(buf)
            // return Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "CMYK32 nie jest bezpośrednio wspierany przez DynamicImage"));
        },
    };

    Ok(obraz)
}