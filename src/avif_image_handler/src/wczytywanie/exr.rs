use exr::prelude::*;
use image::{DynamicImage, Rgba32FImage};
use std::io::Cursor;

/// # Decoding Exr
/// Some use of libheif_rs.
pub fn exr_loading(bajty: &[u8]) -> std::result::Result<DynamicImage, std::io::Error> {

    let kursor = Cursor::new(bajty);

    // Odczytaj pierwszą płaską warstwę ze wszystkimi kanałami
    let image = read()
        .no_deep_data()
        .largest_resolution_level()
        .all_channels()                     // wczytaj wszystkie kanały bez narzucania ich typu
        .all_layers().all_attributes()      // wszystkie warstwy + atrybuty
        .from_buffered(kursor)
        .map_err(|e| std::io::Error::other(format!("Błąd dekodowania EXR: {}", e)))?;

    let layers = image.layer_data;
    // Wybieramy pierwszą warstwę (zazwyczaj jest tylko jedna)
    let layer = layers
        .into_iter()
        .next()
        .ok_or_else(|| std::io::Error::other("Plik EXR nie zawiera żadnej warstwy"))?;

    let size = layer.size;
    let width = size.x() as u32;
    let height = size.y() as u32;
    let channels = layer.channel_data; // AnyChannels<FlatSamples>

    // Znajdź kanały RGBA
    let mut r_samples = None;
    let mut g_samples = None;
    let mut b_samples = None;
    let mut a_samples = None;

    for channel in &channels.list {   // &Vec<AnyChannel<FlatSamples>> → możemy pożyczyć
        let r = Text::from("R");
        let g = Text::from("G");
        let b = Text::from("B");
        let a = Text::from("A");

        match channel.name.clone() {
            name if name == r => r_samples = Some(&channel.sample_data),
            name if name == g => g_samples = Some(&channel.sample_data),
            name if name == b  => b_samples = Some(&channel.sample_data),
            name if name == a => a_samples = Some(&channel.sample_data),
            _ => {}
        }
    }

    let r = r_samples.ok_or_else(|| std::io::Error::other("Brak kanału R"))?;
    let g = g_samples.ok_or_else(|| std::io::Error::other("Brak kanału G"))?;
    let b = b_samples.ok_or_else(|| std::io::Error::other("Brak kanału B"))?;
    let a = a_samples; // opcjonalny

    // Sprawdzamy typ próbek na podstawie kanału R (zakładamy, że wszystkie są tego samego typu)


    let num_pixels = (width * height) as usize;
    let mut rgba_f32 = vec![0.0f32; num_pixels * 4];   // tymczasowy bufor f32

    // Funkcja kopiująca dane kanału do bufora RGBA (składowa: 0=R, 1=G, 2=B, 3=A)
    /// Copy data into RGBA channel
    fn copy_channel(samples: &FlatSamples, rgba: &mut [f32], component: usize, _num_pixels: usize) {
        match samples {
            FlatSamples::F16(data) => {
                for (i, v) in data.iter().enumerate() {
                    rgba[i * 4 + component] = v.to_f32();
                }
            }
            FlatSamples::F32(data) => {
                rgba[component..]
                    .iter_mut()
                    .step_by(4)
                    .zip(data.iter())
                    .for_each(|(dest, &src)| *dest = src);
            }
            FlatSamples::U32(data) => {
                for (i, &v) in data.iter().enumerate() {
                    rgba[i * 4 + component] = v as f32;
                }
            }
        }
    }

    copy_channel(r, &mut rgba_f32, 0, num_pixels);
    copy_channel(g, &mut rgba_f32, 1, num_pixels);
    copy_channel(b, &mut rgba_f32, 2, num_pixels);

    if let Some(a_samples) = a {
        copy_channel(a_samples, &mut rgba_f32, 3, num_pixels);
    } else {
        // jeśli brak kanału alfa – ustaw 1.0
        rgba_f32.iter_mut().skip(3).step_by(4).for_each(|v| *v = 1.0);
    }

    // Tworzymy DynamicImage w zależności od oryginalnej głębi bitowej
    let bufor = Rgba32FImage::from_raw(width, height, rgba_f32)
        .ok_or_else(|| std::io::Error::other("Nie udało się utworzyć Rgba32FImage"))?;



    Ok(DynamicImage::ImageRgba32F(bufor))
}