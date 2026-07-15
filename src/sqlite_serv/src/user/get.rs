use crate::auth::claims::Claims;
use crate::auth::permissions::check_is_admin;
use crate::sql::AppState;
use crate::user::User;
use axum::extract::State;
use axum::Json;
use http::StatusCode;
use sqlx::SqlitePool;

pub async fn handler_user_get_list(
    State(state): State<AppState>,
    claims: Claims,
) -> Result<(StatusCode, Json<Vec<User>>), (StatusCode, String)> {
    println!("Odebrano żądanie get_user_list");
    check_is_admin(&claims)?;
    let user = get_user_list(&state.db)
        .await
        .map_err(|e| {
            eprintln!("Błąd bazy danych przy pobieraniu listy: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    // Jeśli wszystko poszło dobrze, zwracamy status 200 i listę zapakowaną w JSON
    Ok((StatusCode::OK, Json(user)))
}
pub async fn get_user_list(pool: &SqlitePool) -> Result<Vec<User>, sqlx::Error> {
    let users = sqlx::query_as::<_, User>("SELECT * FROM users")
        .fetch_all(pool)
        .await?;

    Ok(users)
}
pub async fn handler_get_user_data_by_id(
    State(state): State<AppState>,
    claims: Claims,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Result<(StatusCode, Json<User>), (StatusCode, String)> {
    println!("Odebrano żądanie get_products_list");
    // if claims.role != "Admin" {
    //     return Err((
    //         StatusCode::FORBIDDEN,
    //         "Brak uprawnień. Ta operacja wymaga roli Admin.".to_string(),
    //     ));
    // }
    check_is_admin(&claims)?;
    let user_data = get_user_data_by_id(id, &state.db)
        .await
        .map_err(|e| {
            eprintln!("Błąd bazy danych przy pobieraniu listy: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    // Jeśli wszystko poszło dobrze, zwracamy status 200 i listę zapakowaną w JSON
    Ok((StatusCode::OK, Json(user_data)))
}
pub async fn handler_get_user_own_data(
    State(state): State<AppState>,
    claims: Claims,
) -> Result<(StatusCode, Json<User>), (StatusCode, String)> {
    println!("Odebrano żądanie get_products_list");

    let user_data = get_user_data_by_id(claims.sub, &state.db)
        .await
        .map_err(|e| {
            eprintln!("Błąd bazy danych przy pobieraniu listy: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    // Jeśli wszystko poszło dobrze, zwracamy status 200 i listę zapakowaną w JSON
    Ok((StatusCode::OK, Json(user_data)))
}
pub async fn get_user_data_by_id(id: i64, pool: &SqlitePool) -> Result<User, sqlx::Error> {

    let user = sqlx::query_as::<_, User>(
        "SELECT
            id,
            username,
            name, /* Zgodnie z Twoją wcześniejszą logiką */
            NULL AS email,    /* Jeśli nie ma tego w bazie */
            password_hash,
            permission,
        CASE WHEN valid = 'true' THEN 1 ELSE 0 END AS valid
         FROM users
         WHERE id = ?"
    )
        .bind(id)
        .fetch_one(pool)
        .await?;

    Ok(user)
}

pub async fn get_user_by_username(
    pool: &SqlitePool,
    username: &str
) -> Result<Option<User>, sqlx::Error> {

    // Zamiast query(), używamy query_as()!
    let user = sqlx::query_as::<_, User>(
        // Wybieramy dokładnie te kolumny, których potrzebujemy.
        // Używamy aliasów (np. username AS name), jeśli nazwy w bazie
        // różnią się od nazw pól w strukturze.
        "SELECT
            id,
            username,
            name, /* Zgodnie z Twoją wcześniejszą logiką */
            NULL AS email,    /* Jeśli nie ma tego w bazie */
            password_hash,
            permission,
        CASE WHEN valid = 'true' THEN 1 ELSE 0 END AS valid
         FROM users
         WHERE username = ?"
    )
        .bind(username)
        .fetch_optional(pool)
        .await?;

    Ok(user)
}