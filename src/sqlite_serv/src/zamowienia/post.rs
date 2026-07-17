use crate::auth::claims::Claims;
use crate::odleglosci_mapa::oblicz_odleglosc_do_klienta;
use crate::zamowienia::{generate_fv_number, CaloscioweZamowienie, Zamowienie, ZamowieniePozycja};
use crate::{AppState, FRONT_SERV_ADRESS};
use axum::extract::State;
use axum::Json;
use http::StatusCode;
use sqlx::SqlitePool;

async fn get_payment_redirect_url() -> Result<String, (StatusCode, String)> {
    let url = format!("{}index.html",FRONT_SERV_ADRESS.get().unwrap_or(&String::new()));

    Ok(url)
}
// #[axum::debug_handler]
pub async fn handle_put_order_new(
    State(state): State<AppState>,
    maybe_claims: Claims,
    Json(mut payload): Json<CaloscioweZamowienie>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, String)> {

    // println!("zebrane dane do zamowienia: {:?}", payload);


    println!("DEBUG: Token sub b4 teraz claims nie maybeclaims (user_id) = {:?}", maybe_claims.sub);
    payload.dane.user_id = Some(maybe_claims.sub);
    println!("DEBUG: Token sub (user_id) = {:?}", payload.dane.user_id);
    println!("zebrane dane do zamowienia: {:?}", payload);

    let (produkty_netto, produkty_vat) = payload.przedmioty.iter()
        .fold((0.0, 0.0), |(acc_netto, acc_vat), item| {
            let netto = item.cena as f64 * item.ilosc as f64;
            let vat_kwota = netto * (item.vat as f64 / 100.0);

            (acc_netto + netto, acc_vat + vat_kwota)
        });
// todo!() ogarnąć żeby było na froncie podgląd kwoty za transport (osobne wywołanie)
    let ulica = &payload.dane.lokacja.ulica;
    let miasto = &payload.dane.lokacja.miasto;
    let kod_pocztowy = &payload.dane.lokacja.kod_pocztowy;


    let kwota_za_trase = match oblicz_odleglosc_do_klienta(ulica, miasto, kod_pocztowy).await {
        Ok(km) => {
            println!("Wyznaczono trasę: {} km", km.odleglosc_km);
            Some(km) // Zapiszemy to w bazie
        }
        Err(e) => {
            eprintln!("Błąd wyznaczania trasy: {}. Zapisuję bez transportu.", e);
            None // W razie błędu zapisujemy jako brak transportu lub domyślną wartość
        }
    };
    let calkowita_kwota_netto: f32 = kwota_za_trase.as_ref().map(|t| t.cena_netto).unwrap_or(0.0) + produkty_netto as f32;
    let calkowita_kwota_vat: f32 = kwota_za_trase.as_ref().map(|t| t.stawka_vat).unwrap_or(0.23) + produkty_vat as f32;

    let dane_trasy = kwota_za_trase;

    let nowe_zamowienie = Zamowienie::new(
        payload.dane.user_id,
        payload.dane.email,
        payload.dane.tel,
        payload.dane.lokacja,
        payload.dane.faktura_dane,
        dane_trasy,
        payload.dane.imie,
        payload.dane.nazwisko,
        calkowita_kwota_netto,
        calkowita_kwota_vat,
        &state.db,
    ).await;

    put_order_new(&state.db, &nowe_zamowienie, &payload.przedmioty)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Pobieramy URL (Twoja funkcja wydmuszka)
    let url = get_payment_redirect_url().await?;

    // Zwracamy JSON zamiast samego kodu statusu
    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "payment_url": url }))
    ))
}
pub async fn put_order_new(
    pool: &SqlitePool,
    new_order: &Zamowienie,
    items: &[ZamowieniePozycja]
) -> Result<(), sqlx::Error> {
    // Rozpocznij transakcję
    let mut tx = pool.begin().await?;
    let numer_fv = generate_fv_number(pool).await?;
    // Wstawienie zamówienia
    println!("nowe zamówienie");
    let result = sqlx::query(
        r#"INSERT INTO orders (
            user_id, date, imie, nazwisko, email, tel, ulica, miasto, kod_pocztowy,
            nazwa_firmy, nip, fv_ulica, fv_miasto, fv_kod_pocztowy,
            odleglosc_km, cena_netto, transport_stawka_vat,
            cena, vat, numer_fv, oplacone
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#
    )
        .bind(new_order.user_id)
        .bind(&new_order.date)
        .bind(&new_order.imie)
        .bind(&new_order.nazwisko)
        .bind(&new_order.email)
        .bind(&new_order.tel)
        // Lokacja
        .bind(&new_order.lokacja.ulica)
        .bind(&new_order.lokacja.miasto)
        .bind(&new_order.lokacja.kod_pocztowy)
        // Faktura (Option)
        .bind(new_order.faktura_dane.as_ref().map(|f| &f.nazwa_firmy))
        .bind(new_order.faktura_dane.as_ref().map(|f| &f.nip))
        .bind(new_order.faktura_dane.as_ref().map(|f| &f.ulica))
        .bind(new_order.faktura_dane.as_ref().map(|f| &f.miasto))
        .bind(new_order.faktura_dane.as_ref().map(|f| &f.kod_pocztowy))
        // Transport (Option)
        .bind(new_order.transport.as_ref().map(|t| t.odleglosc_km))
        .bind(new_order.transport.as_ref().map(|t| t.cena_netto))
        .bind(new_order.transport.as_ref().map(|t| t.stawka_vat))
        // Reszta
        .bind(new_order.cena)
        .bind(new_order.vat)
        .bind(&numer_fv)
        .bind(new_order.oplacone)
        .execute(&mut *tx)
        .await?;
    println!("przeszło przez querry");
    let order_id = result.last_insert_rowid();

    // 2. Wstawienie pozycji (pętla)
    for item in items {
        sqlx::query(
            "INSERT INTO orders_things (zamowienie_id, product_id, ilosc, cena, vat, konfiguracja) VALUES (?, ?, ?, ?, ?, ?)"
        )
            .bind(order_id)
            .bind(item.product_id)
            .bind(item.ilosc)
            .bind(item.cena)
            .bind(item.vat)
            .bind(&item.konfiguracja)
            .execute(&mut *tx)
            .await?;
    }
    println!("już na końcu ;)");

    tx.commit().await?;
    Ok(())
}