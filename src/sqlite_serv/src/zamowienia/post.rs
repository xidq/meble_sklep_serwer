use axum::extract::State;
use axum::Json;
use http::StatusCode;
use sqlx::SqlitePool;
use crate::auth::claims::{Claims, OptionalClaims};
use crate::sql::AppState;
use crate::zamowienia::{CaloscioweZamowienie, Zamowienie, ZamowieniePozycja};

async fn get_payment_redirect_url() -> Result<String, (StatusCode, String)> {
    let url = "http://localhost:8081/index.html".to_string();

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

    let nowe_zamowienie = Zamowienie::new(
        payload.dane.user_id,
        payload.dane.email,
        payload.dane.tel,
        payload.dane.lokacja,
        payload.dane.faktura_dane,
        payload.dane.cena,
        &state.db
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

    // 1. Wstawienie zamówienia
    let result = sqlx::query(
        r#"INSERT INTO orders (user_id, date, email, tel, ulica, miasto, kod_pocztowy,
                               nazwa_firmy, nip, fv_ulica, fv_miasto, fv_kod_pocztowy,
                               cena, numer_fv, oplacone)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#
    )
        .bind(new_order.user_id.unwrap())
        .bind(&new_order.date)
        .bind(&new_order.email)
        .bind(&new_order.tel)
        .bind(&new_order.lokacja.ulica)
        .bind(&new_order.lokacja.miasto)
        .bind(&new_order.lokacja.kod_pocztowy)
        .bind(new_order.faktura_dane.as_ref().map(|f| &f.nazwa_firmy))
        .bind(new_order.faktura_dane.as_ref().map(|f| &f.nip))
        .bind(new_order.faktura_dane.as_ref().map(|f| &f.ulica))
        .bind(new_order.faktura_dane.as_ref().map(|f| &f.miasto))
        .bind(new_order.faktura_dane.as_ref().map(|f| &f.kod_pocztowy))
        .bind(new_order.cena)
        .bind(&new_order.numer_fv)
        .bind(new_order.oplacone)
        .execute(&mut *tx)
        .await?;

    let order_id = result.last_insert_rowid();

    // 2. Wstawienie pozycji (pętla)
    for item in items {
        sqlx::query(
            "INSERT INTO orders_things (zamowienie_id, product_id, ilosc, cena, konfiguracja) VALUES (?, ?, ?, ?, ?)"
        )
            .bind(order_id)
            .bind(item.product_id)
            .bind(item.ilosc)
            .bind(item.cena)
            .bind(&item.konfiguracja)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;
    Ok(())
}