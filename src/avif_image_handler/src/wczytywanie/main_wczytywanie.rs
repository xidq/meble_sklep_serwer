use crate::wczytywanie::avif::avif;
use crate::wczytywanie::check_extension::rozpoznaj_format;
use crate::wczytywanie::ff::ff;
use crate::wczytywanie::jpg::jpeg;
use crate::wczytywanie::png::png;
use crate::wczytywanie::qoi::qoi;
use crate::wczytywanie::tga::tga;
use crate::wczytywanie::unknown::unknown;
use crate::wczytywanie::webp::webp;
use std::io::Read;
use std::path::PathBuf;
use image::DynamicImage;
use crate::wczytywanie::exr::exr_loading;
use strum::{Display, EnumIter, EnumMessage};

#[derive(Clone, Debug, PartialEq, EnumIter, EnumMessage, Display)]
pub enum ImgExtTag {
    #[strum(message = "Jpg", detailed_message = "Joint Photographic Experts Group")]
    Jpg,
    #[strum(message = "Png", detailed_message = "Portable Network Graphics")]
    Png,
    #[strum(message = "Webp", detailed_message = "Web Photograph")]
    Webp,
    #[strum(message = "Tga", detailed_message = "Truevision TGA")]
    Tga,
    #[strum(message = "FF", detailed_message = "Farbfeld")]
    Ff,
    #[strum(message = "Qoi", detailed_message = "Quite OK Image Format")]
    Qoi,
    #[strum(message = "Avif", detailed_message = "AV1 Image File Format")]
    Avif,
    #[strum(message = "Exr", detailed_message = "OpenEXR")]
    Exr,
    #[strum(message = "Unknown", detailed_message = "Unknown")]
    Unknown,
}
/// # Main fn for image decoding
/// That's how it is.
/// 
/// I'm using Image crate as just wrapper for moving data,
/// and using their resize fn elsewhere ;)
pub fn wczytaj_pliki(
    ścieżka: PathBuf
) -> Result<(DynamicImage, String), std::io::Error>{
    let bajty = std::fs::read(&ścieżka)?;
    let nazwa = ścieżka
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("nieznany")
        .to_string();
    let rozszerzenie = ścieżka
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    // let sprawdzanie_kompresji = sprawdzanie_kompresji_zdjecia?;
    let sprawdzanie_kompresji = match rozszerzenie.as_str(){
        "zst" => {
            let mut decoder = zstd::stream::read::Decoder::new(&bajty[..])?;
            let mut rozpakowane = Vec::new();
            decoder.read_to_end(&mut rozpakowane)?;
            rozpakowane
        }
        "bz2" => {
            let mut decoder = bzip2::read::BzDecoder::new(&bajty[..]);
            let mut rozpakowane = Vec::new();
            decoder.read_to_end(&mut rozpakowane)?;
            rozpakowane
        }
        "xz" => {
            let mut decoder = xz2::read::XzDecoder::new(&bajty[..]);
            let mut rozpakowane = Vec::new();
            decoder.read_to_end(&mut rozpakowane)?;
            rozpakowane
        }
        _ => {bajty}
    };
    
    let fotu= match rozpoznaj_format(&sprawdzanie_kompresji){
        ImgExtTag::Avif => {avif(&sprawdzanie_kompresji)}
        ImgExtTag::Jpg => {jpeg(&sprawdzanie_kompresji)}
        ImgExtTag::Png => {png(&sprawdzanie_kompresji)}
        ImgExtTag::Webp => {webp(&sprawdzanie_kompresji)}
        ImgExtTag::Tga => {tga(&sprawdzanie_kompresji)}
        ImgExtTag::Ff => {ff(&sprawdzanie_kompresji)}
        ImgExtTag::Qoi => {qoi(&sprawdzanie_kompresji)}
        ImgExtTag::Unknown => {unknown(&sprawdzanie_kompresji)}
        ImgExtTag::Exr => {exr_loading(&sprawdzanie_kompresji)}
    }?;
    
    Ok((fotu, nazwa))
}