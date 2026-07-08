use axum::extract::State;
use axum::Json;
use http::StatusCode;
use crate::sql::{AppState, Product};

pub async fn list_products(State(state): State<AppState>) -> (StatusCode, Json<Vec<Product>>) {
    // Wykonanie zapytania do bazy zamiast czytania z pliku
    let products = sqlx::query_as::<_, Product>("SELECT * FROM products")
        .fetch_all(&state.db)
        .await;
    match products {
        Ok(list) => (StatusCode::OK, Json(list)),
        Err(e) => {
            eprintln!("Błąd bazy danych: {}", e); // To Ci powie, co dokładnie nie działa
            (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![]))
        }
    }

}