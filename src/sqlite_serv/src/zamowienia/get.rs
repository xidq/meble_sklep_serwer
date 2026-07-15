use axum::extract::State;
use axum::Json;
use http::StatusCode;
use sqlx::Row;
use sqlx::sqlite::SqliteRow;
use crate::auth::claims::Claims;
use crate::AppState;
use crate::zamowienia::{Zamowienie, ZamowienieFV, ZamowienieLokacja};

pub async fn handler_get_user_orders(
    State(state): State<AppState>,
    claims: Claims, // Pobieramy ID zalogowanego użytkownika
) -> Result<Json<Vec<Zamowienie>>, (StatusCode, String)> {
    let rows = sqlx::query("SELECT * FROM orders WHERE user_id = ?")
        .bind(claims.sub)
        .fetch_all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let orders = rows.into_iter().map(|row: SqliteRow| {
        Zamowienie {
            id: row.get("id"),
            user_id: row.get("user_id"),
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
            cena: row.get::<f64, _>("cena") as f32,
            numer_fv: row.get("numer_fv"),
            oplacone: row.get::<i32, _>("oplacone") != 0,
        }
    }).collect();

    Ok(Json(orders))
}