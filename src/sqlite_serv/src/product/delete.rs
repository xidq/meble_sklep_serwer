use crate::sql::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use sqlx::SqlitePool;

pub async fn handler_delete_product_by_id(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Result<StatusCode, (StatusCode, String)> {
    println!("Odebrano żądanie delete_product_by_id dla id: {}", id);

    // Wywołujemy funkcję SQL i mapujemy ogólne błędy bazy danych
    let rows_affected = delete_product_by_id(&state.db, id)
        .await
        .map_err(|e| {
            // Sprawdzamy, czy to blokada przez klucz obcy (ON DELETE RESTRICT dla zamówień)
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.is_foreign_key_violation() {
                    return (StatusCode::CONFLICT, "Nie można usunąć produktu, ponieważ znajduje się w zamówieniach.".to_string());
                }
            }
            eprintln!("Błąd bazy danych przy usuwaniu: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    // Jeśli baza nie usunęła żadnego wiersza, znaczy że nie było takiego produktu
    if rows_affected == 0 {
        return Err((StatusCode::NOT_FOUND, "Produkt o podanym ID nie istnieje".to_string()));
    }

    // Sukces - usunięto, zwracamy 204 No Content
    Ok(StatusCode::NO_CONTENT)
}
pub async fn delete_product_by_id(pool: &SqlitePool, product_id: i64) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM products WHERE id = ?")
        .bind(product_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected())

}