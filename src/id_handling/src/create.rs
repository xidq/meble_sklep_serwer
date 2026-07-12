use crate::enums_structs::ElementyJson;
use http::StatusCode;

pub fn get_new_id<T>(mut typ: T, produkty: &mut Vec<T>) ->
                                                         // Result<(),axum::response::Response>
                                                         Result<(), (StatusCode, &'static str)>
where T: ElementyJson
{
    if typ.get_id() == 0 {
        let prefix = typ.get_value() as i64;

        let base_multiplier = 1_000_000_000;
        let base_id = prefix * base_multiplier;

        let next_id = produkty
            .iter()
            .map(|p| p.get_id())
            .filter(|&id| id / base_multiplier == prefix)
            .max()
            .unwrap_or(base_id) + 1;

        typ.set_id(next_id);
        produkty.push(typ);
    } else {
        if let Some(existing) = produkty.iter_mut().find(|p| p.get_id() == typ.get_id()) {
            *existing = typ;
        } else {
            // return Err((http::status::StatusCode::NOT_FOUND, "Produkt o podanym ID nie istnieje w bazie").into_response());
            return Err((
                StatusCode::NOT_FOUND,
                "Produkt o podanym ID nie istnieje w bazie"
            ));
        }
    }
    Ok(())
}