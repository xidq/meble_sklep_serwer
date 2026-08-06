use crate::AppState;
use crate::auth::claims::Claims;
use crate::auth::permissions::check_is_admin;
use crate::zamowienia::{CaloscioweZamowienie, Pieniadze, Waluta};
use axum::Json;
use axum::extract::State;
use http::StatusCode;

pub async fn handle_admin_edit_orders(
    State(state): State<AppState>,
    claims: Claims,
    Json(zamowienie): Json<CaloscioweZamowienie<f64>>,
) -> Result<StatusCode, (StatusCode, String)> {
    println!("Odebrano żądanie zmiany zamowienia id: {}", zamowienie.dane.id);

    check_is_admin(&claims)?;

    let cena_pieniadze = Pieniadze::new(zamowienie.dane.cena, Waluta::Pln);
    let vat_pieniadze = Pieniadze::new(zamowienie.dane.vat, Waluta::Pln);

    // let dane = &zamowienie.dane;
    // let f_dane = dane.faktura_dane.as_ref();
    // let transport = dane.transport.as_ref();
    // let f_nazwa = f_dane.map(|f| &f.nazwa_firmy);
    // let f_nip = f_dane.map(|f| &f.nip);
    // let f_ulica = f_dane.and_then(|f| f.ulica.as_deref());
    // let f_miasto = f_dane.and_then(|f| f.miasto.as_deref());
    // let f_kod = f_dane.and_then(|f| f.kod_pocztowy.as_deref());

    sqlx::query(
        r#"
        UPDATE orders
        SET
            user_id = ?,
            date = ?,
            imie = ?,
            nazwisko = ?,
            email = ?,
            tel = ?,
            ulica = ?,
            miasto = ?,
            kod_pocztowy = ?,
            nazwa_firmy = ?,
            nip = ?,
            fv_ulica = ?,
            fv_miasto = ?,
            fv_kod_pocztowy = ?,
            odleglosc_km = ?,
            cena_netto = ?,
            transport_stawka_vat = ?,
            cena_dziesiatki = ?,
            cena_grosze = ?,
            vat_dziesiatki = ?,
            vat_grosze = ?,
            waluta = ?,
            numer_fv = ?,
            oplacone = ?,
            status = ?
        WHERE id = ?
        "#
    )
        .bind(zamowienie.dane.user_id)
        .bind(&zamowienie.dane.date)
        .bind(&zamowienie.dane.imie)
        .bind(&zamowienie.dane.nazwisko)
        .bind(&zamowienie.dane.email)
        .bind(&zamowienie.dane.tel)
        // Lokacja dostawy
        .bind(&zamowienie.dane.lokacja.ulica)
        .bind(&zamowienie.dane.lokacja.miasto)
        .bind(&zamowienie.dane.lokacja.kod_pocztowy)
        // Dane faktury (Option)
        .bind(zamowienie.dane.faktura_dane.as_ref().map(|f| &f.nazwa_firmy))
        .bind(zamowienie.dane.faktura_dane.as_ref().map(|f| &f.nip))
        .bind(zamowienie.dane.faktura_dane.as_ref().and_then(|f| f.ulica.as_ref()))
        .bind(zamowienie.dane.faktura_dane.as_ref().and_then(|f| f.miasto.as_ref()))
        .bind(zamowienie.dane.faktura_dane.as_ref().and_then(|f| f.kod_pocztowy.as_ref()))
        // Dane transportu (Option)
        .bind(zamowienie.dane.transport.as_ref().map(|t| t.odleglosc_km))
        .bind(zamowienie.dane.transport.as_ref().map(|t| t.cena_netto))
        .bind(zamowienie.dane.transport.as_ref().map(|t| t.stawka_vat))
        // Kwoty rozbite na Pieniadze
        .bind(cena_pieniadze.dziesiatki)
        .bind(cena_pieniadze.grosze)
        .bind(vat_pieniadze.dziesiatki)
        .bind(vat_pieniadze.grosze)
        .bind(Waluta::Pln.get_name())
        // Statusy i numery
        .bind(&zamowienie.dane.numer_fv)
        .bind(&zamowienie.dane.oplacone)
        .bind(&zamowienie.dane.status)
        // WHERE id = ?
        .bind(zamowienie.dane.id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::OK)
}