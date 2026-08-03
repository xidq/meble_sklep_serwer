use crate::auth::claims::Claims;
use crate::odleglosci_mapa::oblicz_odleglosc_do_klienta;
use crate::zamowienia::{generate_fv_number, CaloscioweZamowienie, Pieniadze, Waluta, Zamowienie, ZamowieniePozycja};
use crate::AppState;
use axum::extract::State;
use axum::Json;
use http::StatusCode;
use sqlx::SqlitePool;
use env_thingy::FRONT_SERV_ADRESS;

async fn get_payment_redirect_url() -> Result<String, (StatusCode, String)> {
    let url = format!("{}index.html",FRONT_SERV_ADRESS.get().unwrap_or(&String::new()));

    Ok(url)
}
// #[axum::debug_handler]
pub async fn handle_put_order_new(
    State(state): State<AppState>,
    maybe_claims: Claims,
    Json(mut payload): Json<CaloscioweZamowienie<f64>>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, String)> {

    // println!("zebrane dane do zamowienia: {:?}", payload);


    println!("DEBUG: Token sub b4 teraz claims nie maybeclaims (user_id) = {:?}", maybe_claims.sub);
    payload.dane.user_id = Some(maybe_claims.sub);
    println!("DEBUG: Token sub (user_id) = {:?}", payload.dane.user_id);
    println!("zebrane dane do zamowienia: {:?}", payload);

    let (produkty_netto, produkty_vat) = payload.przedmioty.iter()
        .fold((0.0, 0.0), |(acc_netto, acc_vat), item| {
            let netto = item.cena * item.ilosc as f64;
            let vat_kwota = netto * (item.vat / 100.0);

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
    let calkowita_kwota_netto: f64 = kwota_za_trase.as_ref().map(|t| t.cena_netto).unwrap_or(0.0) + produkty_netto;
    let calkowita_kwota_vat: f64 = kwota_za_trase.as_ref().map(|t| t.stawka_vat).unwrap_or(0.23) + produkty_vat;

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
        Pieniadze::new(calkowita_kwota_netto, Waluta::Pln),
        Pieniadze::new(calkowita_kwota_vat, Waluta::Pln),
        &state.db,
    ).await;
    let zmapowane_pozycje: Vec<ZamowieniePozycja<Pieniadze>> = payload.przedmioty.into_iter().map(|item| {
        ZamowieniePozycja {
            zamowienie_id: 0,
            product_id: item.product_id,
            ilosc: item.ilosc,
            cena: Pieniadze::new(item.cena, Waluta::Pln),
            vat: Pieniadze::new(item.vat, Waluta::Pln), // lub traktowane jako stawka procentowa/kwotowa w zależności od logiki
            konfiguracja: item.konfiguracja,
        }
    }).collect();

    put_order_new(&state.db, &nowe_zamowienie, &zmapowane_pozycje)
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
    new_order: &Zamowienie<Pieniadze>,
    items: &[ZamowieniePozycja<Pieniadze>]
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
            cena_dziesiatki,cena_grosze, vat_dziesiatki, vat_grosze, waluta, numer_fv, oplacone, status
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#
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
        .bind(new_order.cena.dziesiatki)
        .bind(new_order.cena.grosze)
        .bind(new_order.vat.dziesiatki)
        .bind(new_order.vat.grosze)
        .bind(new_order.vat.waluta.get_name())
        .bind(&numer_fv)
        .bind(&new_order.oplacone)
        .bind(&new_order.status)
        .execute(&mut *tx)
        .await?;
    println!("przeszło przez querry");
    let order_id = result.last_insert_rowid();

    // 2. Wstawienie pozycji (pętla)
    for item in items {
        sqlx::query(
            "INSERT INTO orders_things (
                           zamowienie_id, product_id, ilosc,
                           cena_dziesiatki,cena_grosze, vat_dziesiatki, vat_grosze, waluta,
                           konfiguracja) VALUES (?, ?, ?, ?, ?, ?,? ,? ,?)"
        )
            .bind(order_id)
            .bind(item.product_id)
            .bind(item.ilosc)
            .bind(item.cena.dziesiatki)
            .bind(item.cena.grosze)
            .bind(item.vat.dziesiatki)
            .bind(item.vat.grosze)
            .bind(item.cena.waluta.get_name())
            .bind(&item.konfiguracja)
            .execute(&mut *tx)
            .await?;
    }
    println!("już na końcu ;)");

    tx.commit().await?;
    Ok(())
}