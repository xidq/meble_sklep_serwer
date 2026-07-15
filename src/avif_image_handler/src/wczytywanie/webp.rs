use image::DynamicImage;
use webp::Decoder;

/// # Decoding webp
/// ;)
pub fn webp(bajty: &[u8]) -> Result<DynamicImage, std::io::Error> {

    let decoder = Decoder::new(bajty);

    let obraz = match decoder.decode() {
        Some(img) => img.to_image(),
        None => {

            image::load_from_memory_with_format(bajty, image::ImageFormat::WebP)
                .map_err(|e| std::io::Error::other(format!("Błąd dekodowania klatki WebP: {}", e)))?
        }
    };

    Ok(obraz)
}