use axum::extract::State;
use http::StatusCode;
use sqlx::SqlitePool;
use crate::auth::claims::Claims;
use crate::sql::AppState;

pub async fn handler_delete_user_by_user(
    State(state): State<AppState>,
    // axum::extract::Path(id): axum::extract::Path<i64>,
    claims: Claims,
) -> Result<StatusCode, (StatusCode, String)> {
    // Zamiast zmiennej z Path, podajesz ID bezpośrednio wyciągnięte z tokenu JWT!
    let my_id = claims.sub;

    println!("Użytkownik o ID {} zażądał usunięcia swojego konta", my_id);

    let rows_affected = delete_user_by_id(&state.db, my_id)
        .await
        .map_err(|e| {
            // Twój niezmieniony kod mapowania błędów...
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    if rows_affected == 0 {
        return Err((StatusCode::NOT_FOUND, "Twoje konto nie istnieje".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}
pub async fn handler_delete_user_by_id(
    State(state): State<AppState>,
    claims: Claims,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Result<StatusCode, (StatusCode, String)> {

    println!("Odebrano żądanie delete_user_by_id dla id: {}", id);

    if claims.role != "Admin" {
        return Err((
            StatusCode::FORBIDDEN,
            "Brak uprawnień. Ta operacja wymaga roli Admin.".to_string(),
        ));
    }

    let rows_affected = delete_user_by_id(&state.db, id)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.is_foreign_key_violation() {
                    return (StatusCode::CONFLICT, "Nie można usunąć użytkownika, ponieważ znajduje się w zamówieniach.".to_string());
                }
            }

            eprintln!("Błąd bazy danych przy usuwaniu: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())

        })?;

    if rows_affected == 0 {
        return Err((StatusCode::NOT_FOUND, "User o podanym ID nie istnieje".to_string()));
    }
    // Sukces - usunięto, zwracamy 204 No Content

    Ok(StatusCode::NO_CONTENT)

}
pub async fn delete_user_by_id(pool: &SqlitePool, user_id: i64) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected())

}