use crate::AppState;
use crate::auth::claims::Claims;
use crate::odleglosci_mapa::oblicz_odleglosc_do_klienta;
use crate::zamowienia::{generate_fv_number, CaloscioweZamowienie, Zamowienie, ZamowieniePozycja};
use axum::Json;
use axum::extract::State;
use env_thingy::{OnceLockExt, FRONT_SERV_ADDRESS};
use http::StatusCode;
use rust_decimal::Decimal;
use sqlx::SqlitePool;

async fn get_payment_redirect_url() -> Result<String, (StatusCode, String)> {
    let url = format!("{}index.html", FRONT_SERV_ADDRESS.v(""));

    Ok(url)
}
// #[axum::debug_handler]
pub async fn handle_put_order_new(
    State(state): State<AppState>,
    maybe_claims: Claims,
    Json(mut payload): Json<CaloscioweZamowienie>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, String)> {

    // println!("zebrane dane do zamowienia: {:?}", payload);


    println!(
        "DEBUG: Token sub b4 teraz claims nie maybeclaims (user_id) = {:?}",
        maybe_claims.sub
    );
    payload.dane.user_id = Some(maybe_claims.sub);
    println!("DEBUG: Token sub (user_id) = {:?}", payload.dane.user_id);
    println!("zebrane dane do zamowienia: {:?}", payload);

    let (produkty_netto, produkty_vat) = payload.przedmioty.iter().fold(
        (Decimal::ZERO, Decimal::ZERO),
        |(acc_netto, acc_vat), item| {
            let netto: Decimal = item.cena * Decimal::new(item.ilosc,0);
            let vat_kwota = netto * item.vat;

            (acc_netto + netto, acc_vat + vat_kwota)
        });

    let ulica = &payload.dane.lokacja.ulica;
    let miasto = &payload.dane.lokacja.miasto;
    let kod_pocztowy = &payload.dane.lokacja.kod_pocztowy;


    let kwota_za_trase = match oblicz_odleglosc_do_klienta(ulica, miasto, kod_pocztowy).await {
        Ok(km) => {
            println!("Wyznaczono trasę: {} km", km.odleglosc_km);
            Some(km)
        }
        Err(e) => {
            eprintln!("Błąd wyznaczania trasy: {}. Zapisuję bez transportu.", e);
            None // W razie błędu zapisujemy jako brak transportu lub domyślną wartość
        }
    };
    let calkowita_kwota_netto: Decimal = kwota_za_trase
        .as_ref()
        .map(|t| t.cena_netto).unwrap_or(Decimal::ZERO)
        + produkty_netto;
    let calkowita_kwota_vat: Decimal = kwota_za_trase
        .as_ref()
        .map(|t| t.stawka_vat)
        .unwrap_or(Decimal::new(23, 2))
        + produkty_vat;

    let dane_trasy = kwota_za_trase;

    let nowe_zamowienie = Zamowienie::new()
        .add_user_id(payload.dane.user_id)
        .add_imie(payload.dane.imie)
        .add_nazwisko(payload.dane.nazwisko)
        .add_email(payload.dane.email)
        .add_tel(payload.dane.tel)
        .add_lokacja(payload.dane.lokacja)
        .add_fv(payload.dane.faktura_dane)
        .add_transport(dane_trasy)
        .add_cena(calkowita_kwota_netto)
        .add_vat(calkowita_kwota_vat)
        .generuj_nr_fv(&state.db)
        .await;

    let zmapowane_pozycje: Vec<ZamowieniePozycja> = payload
        .przedmioty
        .into_iter()
        .map(|item| {
        ZamowieniePozycja {
            zamowienie_id: 0,
            product_id: item.product_id,
            ilosc: item.ilosc,
            cena: item.cena,
            vat: item.vat,
            konfiguracja: item.konfiguracja,
        }
    })
        .collect();

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
    new_order: &Zamowienie,
    items: &[ZamowieniePozycja],
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
            cena, vat, waluta, numer_fv, oplacone, status
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#
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
        .bind(new_order.transport.as_ref().map(|t| t.cena_netto.to_string()))
        .bind(new_order.transport.as_ref().map(|t| t.stawka_vat.to_string()))
        // Reszta
        .bind(new_order.cena.to_string())
        .bind(new_order.vat.to_string())
        .bind(new_order.waluta.get_name())
        .bind(&numer_fv)
        .bind(&new_order.oplacone)
        .bind(&new_order.status)
        .execute(&mut *tx)
        .await?;
    println!("przeszło przez querry");
    let order_id = result.last_insert_rowid();

    // Wstawienie pozycji (pętla)
    for item in items {
        sqlx::query(
            "INSERT INTO orders_things (
                           zamowienie_id, product_id, ilosc,
                           cena, vat,
                           konfiguracja) VALUES (?, ?, ?, ?, ?, ?)",
        )
            .bind(order_id)
            .bind(item.product_id)
            .bind(item.ilosc)
            .bind(item.cena.to_string())
            .bind(item.vat.to_string())
            .bind(&item.konfiguracja)
            .execute(&mut *tx)
            .await?;
    }
    println!("już na końcu ;)");

    tx.commit().await?;
    Ok(())
}