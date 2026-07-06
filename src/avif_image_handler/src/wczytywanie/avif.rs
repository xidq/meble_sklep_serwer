use std::io;
use image::{DynamicImage, ImageBuffer, Rgb, Rgba};
use libheif_rs::{ColorSpace, HeifContext, LibHeif, RgbChroma};
/// # Avif decoding
/// Using libheif, mean... Somewhat wrapping...
/// 
/// just using libheif, ok?
/// 
/// Put data from avif to get image, and exif, and color profile...
/// 
/// nice, right?
pub fn avif(bajty: &[u8]) -> Result<DynamicImage, std::io::Error>{
    let lib_heif = LibHeif::new();
    let ctx = HeifContext::read_from_bytes(bajty)
        .map_err(std::io::Error::other)?;
    let handle = ctx.primary_image_handle()
        .map_err(std::io::Error::other)?;

    let has_alpha = handle.has_alpha_channel();
    let bit_depth = handle.luma_bits_per_pixel();

    let (chroma, bytes_per_channel) = if bit_depth > 8 {
        if has_alpha { (RgbChroma::HdrRgbaBe, 2) } else { (RgbChroma::HdrRgbBe, 2) }
    } else {
        if has_alpha { (RgbChroma::Rgba, 1) } else { (RgbChroma::Rgb, 1) }
    };

    let image = lib_heif.decode(
        &handle,
        ColorSpace::Rgb(chroma),
        None,
    ).map_err(std::io::Error::other)?;

    let width = image.width() as usize;
    let height = image.height() as usize;
    let channels = if has_alpha { 4 } else { 3 };

    let planes = image.planes();
    let interleaved = planes.interleaved.ok_or_else(|| {
        std::io::Error::other("Brak danych interleaved")
    })?;

    let data = interleaved.data;
    let stride = interleaved.stride;

    let mut clean_vec = Vec::with_capacity(width * height * channels * bytes_per_channel);

    for y in 0..height {
        let line_start = y * stride;
        let line_end = line_start + (width * channels * bytes_per_channel);

        clean_vec.extend_from_slice(&data[line_start..line_end]);
    }

    
    let obraz = if bit_depth > 8 {

        let data_u16: Vec<u16> = clean_vec
            .chunks_exact(2)
            .map(|c| u16::from_be_bytes([c[0], c[1]]))
            .collect();

        if has_alpha {
            let buf = ImageBuffer::<Rgba<u16>, _>::from_raw(width as u32, height as u32, data_u16)
                .ok_or_else(|| io::Error::other("Błąd tworzenia bufora Rgba16a"))?;
            DynamicImage::ImageRgba16(buf)
        } else {
            let buf = ImageBuffer::<Rgb<u16>, _>::from_raw(width as u32, height as u32, data_u16)
                .ok_or_else(|| io::Error::other("Błąd tworzenia bufora Rgb16"))?;
            DynamicImage::ImageRgb16(buf)
        }
    } else {
        // Standardowe 8 bit
        if has_alpha{
            let buffer = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(width as u32, height as u32, clean_vec)
                .ok_or_else(|| std::io::Error::other("Błąd bufora AVIF 8-bita"))?;
            image::DynamicImage::ImageRgba8(buffer)
        } else {
            let buffer = image::ImageBuffer::<image::Rgb<u8>, _>::from_raw(width as u32, height as u32, clean_vec)
                .ok_or_else(|| std::io::Error::other("Błąd bufora AVIF 8-bit"))?;
            image::DynamicImage::ImageRgb8(buffer)
        }


    };

    Ok(obraz)
}