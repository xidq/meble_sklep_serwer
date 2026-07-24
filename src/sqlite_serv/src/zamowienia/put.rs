
use axum::extract::State;
use axum::Json;
use bcrypt::hash;
use http::StatusCode;
use sqlx::SqlitePool;
use crate::AppState;
use crate::auth::claims::Claims;
use crate::auth::permissions::{check_is_admin,check_is_own_acc};
use crate::user::{match_role,User};
use crate::zamowienia::Zamowienie;

pub async fn handle_admin_edit_orders(
    State(state): State<AppState>,
    claims: Claims,
    Json(zamowienie): Json<Zamowienie>,
) -> Result<StatusCode,(StatusCode,String)> {
    println!("Odebrano żądanie zmiany zamowienia id: {}",id);

    check_is_admin(&claims)?;
    
    
    edit_order(
        &state.db,
        id,
    )
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => (StatusCode::NOT_FOUND,"Product not found".to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR,e.to_string()),
        })?;

    Ok(StatusCode::NO_CONTENT)
}
pub async fn edit_order(pool: &SqlitePool, zamowienie: &Zamowienie) -> Result<(),sqlx::Error> {
    
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
            cena = ?,
            vat = ?,
            numer_fv = ?,
            oplacon = ?e
        WHERE id = ?
        "#
    )
        .bind(&zamowienie.user_id)
        .bind(&zamowienie.date)
        .bind(&zamowienie.imie)
        .bind(&zamowienie.nazwisko)
        .bind(&zamowienie.email)
        .bind(&zamowienie.tel)
        .bind(&zamowienie.lokacja.ulica)
        .bind(&zamowienie.lokacja.miasto)
        .bind(&zamowienie.lokacja.kod_pocztowy)
        .bind(&zamowienie.faktura_dane.unwrap_or(""))
        .bind(&zamowienie.lokacja.ulica)


        .bind(user.id) // To ID z lokacji WHERE
        .execute(pool)
        .await?;

    Ok(())
}