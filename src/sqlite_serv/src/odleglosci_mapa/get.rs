use crate::odleglosci_mapa::oblicz_odleglosc_do_klienta;
use crate::zamowienia::{DaneTransportu, ZamowienieLokacja};
use crate::AppState;
use axum::extract::State;
use axum::Json;
use http::StatusCode;

pub async fn handler_order_distance_to_client(
    State(_state): State<AppState>,
    Json(payload): Json<ZamowienieLokacja>,
) -> Result<Json<DaneTransportu>, (StatusCode, String)> {


    let huehue = oblicz_odleglosc_do_klienta(&payload.ulica, &payload.miasto, &payload.kod_pocztowy)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()));
    match huehue {
        Ok(hue) => {
            Ok(Json(hue))
        }
        Err(e) => {
            Err(e)
        }
    }
}