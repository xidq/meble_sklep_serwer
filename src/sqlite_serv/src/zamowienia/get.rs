use axum::extract::State;
use axum::Json;
use http::StatusCode;
use sqlx::Row;
use sqlx::sqlite::SqliteRow;
use crate::auth::claims::Claims;
use crate::AppState;
use crate::auth::permissions::check_is_admin;
use crate::zamowienia::{AdminZamowieniaListView, DaneTransportu, StatusZamowienia, Zamowienie, ZamowienieFV, ZamowienieLokacja};

pub async fn handler_get_user_orders(
    State(state): State<AppState>,
    claims: Claims,
) -> Result<Json<Vec<Zamowienie<f64>>>, (StatusCode, String)> {
    let rows = sqlx::query("SELECT * FROM orders WHERE user_id = ?")
        .bind(claims.sub)
        .fetch_all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let orders = rows.into_iter().map(|row: SqliteRow| {
        Zamowienie {
            id: row.get("id"),
            user_id: row.get("user_id"),
            imie: row.get("imie"),
            nazwisko: row.get("nazwisko"),
            date: row.get("date"),
            email: row.get("email"),
            tel: row.get("tel"),
            lokacja: ZamowienieLokacja {
                ulica: row.get("ulica"),
                miasto: row.get("miasto"),
                kod_pocztowy: row.get("kod_pocztowy"),
            },
            faktura_dane: row.get::<Option<String>, _>("nazwa_firmy").map(|_| ZamowienieFV {
                nazwa_firmy: row.get("nazwa_firmy"),
                nip: row.get("nip"),
                ulica: row.get("fv_ulica"),
                miasto: row.get("fv_miasto"),
                kod_pocztowy: row.get("fv_kod_pocztowy"),
            }),
            transport: row.get::<Option<f64>, _>("odleglosc_km").map(|odleglosc| DaneTransportu {
                odleglosc_km: odleglosc,
                cena_netto: row.get::<f64, _>("cena_netto"),
                stawka_vat: row.get::<f64, _>("transport_stawka_vat"),
            }),
            cena: { 
                let dziesiatki = row.get::<i64, _>("cena_dziesiatki"); 
                let jednosci = row.get::<i64, _>("cena_grosze");
                let wyjscie = dziesiatki as f64 + (jednosci as f64 / 100.);
                wyjscie
            },
            vat: {
                let dziesiatki = row.get::<i64, _>("vat_dziesiatki");
                let jednosci = row.get::<i64, _>("vat_grosze");
                let wyjscie = dziesiatki as f64 + (jednosci as f64 / 100.);
                wyjscie
            },
            numer_fv: row.get("numer_fv"),
            oplacone: row.get("oplacone"),
            status: row.get("status"),
        }
    }).collect();

    Ok(Json(orders))
}


pub async fn handler_admin_get_order_lists(
    State(state): State<AppState>,
    claims: Claims,
) -> Result<Json<Vec<AdminZamowieniaListView>>, (StatusCode, String)> {

    check_is_admin(&claims)?;
    
    let rows = sqlx::query("SELECT id, user_id, date, cena_dziesiatki, cena_grosze, numer_fv, oplacone FROM orders")
        // .bind(claims.sub)
        .fetch_all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let orders = rows.into_iter().map(|row: SqliteRow| {
        AdminZamowieniaListView {
            id: row.get("id"),
            user_id: row.get("user_id"),
            date: row.get("date"),
            cena: {
                let dziesiatki = row.get::<i64, _>("cena_dziesiatki");
                let jednosci = row.get::<i64, _>("cena_grosze");
                let wyjscie = dziesiatki as f64 + (jednosci as f64 / 100.);
                wyjscie
            },
            numer_fv: row.get("numer_fv"),
            oplacone: row.get("oplacone"),
            status: row.get("status"),
        }
    }).collect();

    Ok(Json(orders))
}