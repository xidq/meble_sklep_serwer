use crate::AppState;
use crate::auth::claims::Claims;
use crate::auth::permissions::check_is_admin;
use crate::zamowienia::{
    get_decimal, AdminZamowieniaListView, CaloscioweZamowienie,
    DaneTransportu, Zamowienie, ZamowienieFV, ZamowienieLokacja, ZamowieniePozycja
};
use axum::extract::State;
use axum::Json;
use http::StatusCode;
use sqlx::sqlite::SqliteRow;
use sqlx::Row;

pub async fn handler_get_user_orders(
    State(state): State<AppState>,
    claims: Claims,
) -> Result<Json<Vec<Zamowienie>>, (StatusCode, String)> {
    let rows = sqlx::query("SELECT * FROM orders WHERE user_id = ?")
        .bind(claims.sub)
        .fetch_all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let orders = rows
        .into_iter()
        .map(|row: SqliteRow| Zamowienie {
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
                cena_netto: get_decimal(&row, "cena_netto"),
                stawka_vat: get_decimal(&row, "transport_stawka_vat"),
            }),
            cena: get_decimal(&row, "cena"),
            vat: get_decimal(&row, "vat"),
            numer_fv: row.get("numer_fv"),
            oplacone: row.get("oplacone"),
            status: row.get("status"),
            waluta: row.get("waluta"),
    })
        .collect();

    Ok(Json(orders))
}


pub async fn handler_admin_get_order_lists(
    State(state): State<AppState>,
    claims: Claims,
) -> Result<Json<Vec<AdminZamowieniaListView>>, (StatusCode, String)> {

    check_is_admin(&claims)?;
    
    let rows = sqlx::query("SELECT id, user_id, date, cena, vat, numer_fv, oplacone, status FROM orders")
        // .bind(claims.sub)
        .fetch_all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let orders = rows.into_iter().map(|row: SqliteRow| {
        AdminZamowieniaListView {
            id: row.get("id"),
            user_id: row.get("user_id"),
            date: row.get("date"),
            cena: get_decimal(&row, "cena"),
            vat: get_decimal(&row, "vat"),
            numer_fv: row.get("numer_fv"),
            oplacone: row.get("oplacone"),
            status: row.get("status"),
        }
    })
        .collect();

    Ok(Json(orders))
}

pub async fn handler_admin_get_order_item_by_id(
    State(state): State<AppState>,
    claims: Claims,
    axum::extract::Path(order_id): axum::extract::Path<i64>,
) -> Result<Json<CaloscioweZamowienie>, (StatusCode, String)> {

    check_is_admin(&claims)?;
    println!("zbieranie zamówienia całego");
    let row_order = sqlx::query(
        r#"
        SELECT
            id, date, email, tel,
            ulica, miasto, kod_pocztowy,
            fv_ulica, fv_miasto, fv_kod_pocztowy, nip, nazwa_firmy,
            odleglosc_km, cena_netto, transport_stawka_vat,
            vat, waluta,
            numer_fv, oplacone, status,
            cena,
            user_id, imie, nazwisko
        FROM orders
        WHERE id = ?
        "#
    )
        .bind(order_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let row = match row_order {
        Some(r) => r,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                "Zamówienie nie zostało znalezione".to_string(),
            ));
        },
    };

    // let vat_dziesiatki: i64 = row.get("vat_dziesiatki");
    // let vat_grosze: u8 = row.get::<i64, _>("vat_grosze") as u8;
    // let vat_f64 = vat_dziesiatki as f64 + (vat_grosze as f64 / 100.0);
    //
    // let cena_dziesiatki: i64 = row.get("cena_dziesiatki");
    // let cena_grosze: u8 = row.get::<i64, _>("cena_grosze") as u8;
    // let cena_f64 = cena_dziesiatki as f64 + (cena_grosze as f64 / 100.0);

    let order_f64 = Zamowienie {
        id: row.get("id"),
        date: row.get("date"),
        email: row.get("email"),
        tel: row.get("tel"),
        lokacja: ZamowienieLokacja {
            ulica: row.get("ulica"),
            miasto: row.get("miasto"),
            kod_pocztowy: row.get("kod_pocztowy"),
        },
        faktura_dane: row.get::<Option<String>, _>("nip").map(|nip| ZamowienieFV {
            nip,
            nazwa_firmy: row.get("nazwa_firmy"),
            ulica: row.get("fv_ulica"),
            miasto: row.get("fv_miasto"),
            kod_pocztowy: row.get("fv_kod_pocztowy"),
        }),
        transport: row.get::<Option<f64>, _>("odleglosc_km").map(|odleglosc_km| DaneTransportu {
            odleglosc_km,
            cena_netto: get_decimal(&row, "cena_netto"),
            stawka_vat: get_decimal(&row, "transport_stawka_vat"),
        }),
        vat: get_decimal(&row, "vat"),
        numer_fv: row.get("numer_fv"),
        oplacone: row.get("oplacone"),
        status: row.get("status"),
        cena: get_decimal(&row, "cena"),
        user_id: row.get("user_id"),
        imie: row.get("imie"),
        nazwisko: row.get("nazwisko"),
        waluta: row.get("waluta"),
    };

    let item_rows = sqlx::query(
        r#"
        SELECT
            zamowienie_id, product_id, ilosc,
            cena,
            vat,
            konfiguracja
        FROM orders_things
        WHERE zamowienie_id = ?
        "#
    )
        .bind(order_id)
        .fetch_all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let przedmioty_f64 = item_rows
        .into_iter()
        .map(|item_row| ZamowieniePozycja {
                zamowienie_id: item_row.get("zamowienie_id"),
                product_id: item_row.get("product_id"),
                ilosc: item_row.get("ilosc"),
                cena: get_decimal(&row, "cena"),
                vat: get_decimal(&row, "vat"),
                konfiguracja: item_row.get("konfiguracja"),
        })
        .collect();

    println!("wysyłanie zamówienia całego na front");
    Ok(Json(CaloscioweZamowienie {
        dane: order_f64,
        przedmioty: przedmioty_f64,
    }))
}