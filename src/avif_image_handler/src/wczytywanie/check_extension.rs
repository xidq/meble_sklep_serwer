use crate::wczytywanie::main_wczytywanie::ImgExtTag;

/// # Check format
/// Checkin' format for further decoding
pub fn rozpoznaj_format(bajty: &[u8]) -> ImgExtTag {
    if bajty.len() < 12 { return ImgExtTag::Unknown; }

    let typ = match &bajty[0..12] {
        [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, ..] => ImgExtTag::Png,

        [0xFF, 0xD8, 0xFF, ..] => ImgExtTag::Jpg,

        [0x71, 0x6F, 0x69, 0x66, ..] => ImgExtTag::Qoi,

        [0x66, 0x61, 0x72, 0x62, 0x66, 0x65, 0x6C, 0x64, ..] => ImgExtTag::Ff,

        [b'R', b'I', b'F', b'F', _, _, _, _, b'W', b'E', b'B', b'P'] => ImgExtTag::Webp,

        [_, _, _, _, b'f', b't', b'y', b'p', b'a', b'v', b'i', b'f'] => ImgExtTag::Avif,
        [0x76, 0x2F, 0x31, 0x01, ..] => ImgExtTag::Exr,
        _ => ImgExtTag::Unknown,
    };
    
    if jest_tga(bajty) {
        return ImgExtTag::Tga;
    }

    typ
}
/// # tga check
/// Due to tga format philosophy we're checking if that's it.
fn jest_tga(bajty: &[u8]) -> bool {
    let len = bajty.len();
    if len < 18 { return false; }

    // stopka (TGA 2.0) - szukamy "TRUEVISION-XFILE."
    // Stopka zaczyna się 26 bajtów przed końcem
    if len >= 26 {
        let stopka = &bajty[len - 26..];
        if &stopka[8..24] == b"TRUEVISION-XFILE" {
            return true;
        }
    }

    // nagłówek (TGA 1.0)
    // bajty[1] to typ mapy kolorów (0 lub 1)
    // bajty[2] to typ obrazu (1, 2, 3, 9, 10, 11)
    let color_map_type = bajty[1];
    let image_type = bajty[2];

    let poprawny_color_map = color_map_type <= 1;
    let poprawny_image_type = matches!(image_type, 1 | 2 | 3 | 9 | 10 | 11);

    // Dodatkowo: TGA zazwyczaj ma 0 w bajcie 0 (długość ID), chyba że ma metadane
    poprawny_color_map && poprawny_image_type
}