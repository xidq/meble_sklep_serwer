use image::DynamicImage;
use libheif_rs::{Channel, ColorSpace, CompressionFormat, EncoderParameterValue, EncoderQuality, HeifContext, Image, LibHeif, RgbChroma};
use std::fs::create_dir_all;
use std::path::Path;

/// # Encoding avif
pub async fn avif_match(
    nazwa: String,
    img: DynamicImage,
    sciezka: &Path,
) -> Result<(), tokio::io::Error> {

    let wymiar = vec![2048, 1024, 512, 256, 128, 64, 32, 16];
    for x in wymiar {
        let reskalowanie = img.resize(
            x,
            x,
            image::imageops::FilterType::Lanczos3,
        );

        let nazwa_dodatkowa = match x{
            2048 => "2048",
            1024 => "1024",
            512 => "512",
            256 => "256",
            128 => "128",
            64 => "64",
            32 => "32",
            16 => "16",
            _ => "",
        };


        let heif_img = {

            let (w, h) = reskalowanie.to_rgba8().dimensions();

            let raw_u8: Vec<u8> = reskalowanie.as_bytes().to_vec();

            //
            let mut heif_img = Image::new(w, h, ColorSpace::Rgb(RgbChroma::C444))
                .map_err(std::io::Error::other)?;

            //4 osobne płaszczyzny po 10 bitów każda
            heif_img.create_plane(Channel::R, w, h, 8).expect("Plane R fail");
            heif_img.create_plane(Channel::G, w, h, 8).expect("Plane G fail");
            heif_img.create_plane(Channel::B, w, h, 8).expect("Plane B fail");
            heif_img.create_plane(Channel::Alpha, w, h, 8).expect("Plane A fail");

            {
                let planes = heif_img.planes_mut();

                let stride = planes.r.as_ref().unwrap().stride;
                let data_r = planes.r.unwrap().data;
                let data_g = planes.g.unwrap().data;
                let data_b = planes.b.unwrap().data;
                let data_a = planes.a.unwrap().data;

                // Przechodzimy przez obraz rząd po rzędzie
                for (y, row_src) in raw_u8.chunks_exact(w as usize * 4).enumerate() {
                    let row_offset = y * stride;

                    // Wycinamy plasterki o długości dokładnie 'w' (bo to 1 bajt na piksel)
                    let row_r = &mut data_r[row_offset..row_offset + w as usize];
                    let row_g = &mut data_g[row_offset..row_offset + w as usize];
                    let row_b = &mut data_b[row_offset..row_offset + w as usize];
                    let row_a = &mut data_a[row_offset..row_offset + w as usize];

                    for (x, pixel) in row_src.chunks_exact(4).enumerate() {
                        // Bezpośrednie kopiowanie bajtu 1:1, bez rzutowania i kombinowania
                        row_r[x] = pixel[0];
                        row_g[x] = pixel[1];
                        row_b[x] = pixel[2];
                        row_a[x] = pixel[3];
                    }
                }
            }
            heif_img
        };

        let qual = EncoderQuality::Lossy(90);

        let kompresja = CompressionFormat::Av1;

        let chroma = "420".to_string();


        let lib = LibHeif::new();
        let mut context = HeifContext::new()
            .map_err(std::io::Error::other)?;

        let mut encoder = lib.encoder_for_format(kompresja)
            .map_err(std::io::Error::other)?;

        encoder.set_quality(qual)
            .map_err(std::io::Error::other)?;

        encoder.set_parameter_value("chroma", EncoderParameterValue::String(chroma)).ok();


        encoder.set_parameter_value("speed", EncoderParameterValue::Int(3)).ok(); // 0-10 (wolniej = lepsza kompresja)
        encoder.set_parameter_value("tune", EncoderParameterValue::String("ssim".to_string())).ok(); // Optymalizacja pod jakość wizualną


        context.encode_image(&heif_img, &mut encoder, None).map_err(std::io::Error::other)?;


        let final_bytes = context.write_to_bytes()
            .map_err(std::io::Error::other)?;

        let mut output_path = sciezka.to_path_buf();

        if !output_path.exists() {
            create_dir_all(output_path.clone())?;
        }
        output_path.push(format!("{}_{}.avif",nazwa,nazwa_dodatkowa));

        std::fs::write(&output_path, &final_bytes)
            .map_err(|e| std::io::Error::other(format!("Błąd zapisu pliku: {}", e)))?;

    }
    Ok(())



}

#[cfg(test)]
mod tests {
    use std::io::{Error, ErrorKind};
    use std::path::PathBuf;
    use crate::wczytywanie::main_wczytywanie::wczytaj_pliki;
    use super::*;
    #[tokio::test]
    async fn test_obrobki() -> Result<(), std::io::Error>{


        let path_out = PathBuf::from("../test_data/img_out/");
        let path_in = PathBuf::from("../test_data/img_in/test_1_1.avif");
        if !path_out.exists() {
            return Err(Error::new(ErrorKind::NotFound, "zla sciezka ;("));
        }
        // let nazwa = String::from("test_1_1");
        let mut foto: DynamicImage;
        let mut nazwa: String;
        match wczytaj_pliki(path_in){
            Ok(dane) => { (foto, nazwa ) = dane;}
            Err(e) => return Err(e),
        };
        match avif_match(nazwa, foto, &*path_out).await{
            Ok(_) => {},
            Err(e) => return Err(e),
        };

        Ok(())
    }
}

