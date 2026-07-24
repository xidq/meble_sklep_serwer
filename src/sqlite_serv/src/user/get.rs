use crate::auth::claims::Claims;
use crate::auth::permissions::check_is_admin;
use crate::user::{User, UserData};
use crate::AppState;
use axum::extract::State;
use axum::Json;
use http::StatusCode;
use sqlx::SqlitePool;


/// Admin - get list of all users from database
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

    Ok((StatusCode::OK, Json(user)))
}

/// Gets list of all users in database
pub async fn get_user_list(pool: &SqlitePool) -> Result<Vec<User>, sqlx::Error> {
    let users = sqlx::query_as::<_, User>("SELECT * FROM users")
        .fetch_all(pool)
        .await?;

    Ok(users)
}

/// Admin - get user data by id
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

/// Allow user to get his own data from database
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
pub async fn handler_get_user_profile(
    State(state): State<AppState>,
    claims: Claims,
) -> Result<(StatusCode, Json<UserData>), (StatusCode, String)> {

    let user_data = sqlx::query_as::<_, UserData>(
        "SELECT username, email, name, surname FROM users_data WHERE username = ?"
    )
        .bind(&claims.username)
        .fetch_one(&state.db) // 2. Referencja do bazy (&)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?; // 3. Obsługa błędu

    // 4. Poprawne zwrócenie krotki
    Ok((StatusCode::OK, Json(user_data)))
}
pub async fn get_user_data_by_id(id: i64, pool: &SqlitePool) -> Result<User, sqlx::Error> {

    let user = sqlx::query_as::<_, User>(
        "SELECT
            id,
            username,
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
/// get user data by nick(username)
pub async fn get_user_by_username(
    pool: &SqlitePool,
    username: &str
) -> Result<Option<User>, sqlx::Error> {

    let user = sqlx::query_as::<_, User>(
        "SELECT
            id,
            username,
            name,
            NULL AS email,
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
