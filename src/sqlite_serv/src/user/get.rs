use crate::auth::claims::Claims;
use crate::auth::permissions::check_is_admin;
use crate::user::{User, UserData, UserList};
use crate::AppState;
use axum::extract::State;
use axum::Json;
use http::StatusCode;
use sqlx::SqlitePool;


/// Admin - get list of all users from database
pub async fn handler_user_get_list(
    State(state): State<AppState>,
    claims: Claims,
) -> Result<(StatusCode, Json<Vec<UserList>>), (StatusCode, String)> {
    println!("Odebrano żądanie get_user_list");
    check_is_admin(&claims)?;
    let user = get_user_list(&state.db)
        .await
        .map_err(|e| {
            eprintln!("Błąd bazy danych przy pobieraniu listy użytkowników: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    Ok((StatusCode::OK, Json(user)))
}

/// Gets list of all users in database
pub async fn get_user_list(pool: &SqlitePool) -> Result<Vec<UserList>, sqlx::Error> {
    let users = sqlx::query_as::<_, UserList>("SELECT * FROM users")
        .fetch_all(pool)
        .await?;

    Ok(users)
}

/// Admin - get user data by id
pub async fn handler_get_user_data_by_nameid(
    State(state): State<AppState>,
    claims: Claims,
    axum::extract::Path(name_id): axum::extract::Path<String>,
) -> Result<(StatusCode, Json<UserData>), (StatusCode, String)> {
    println!("Odebrano żądanie get_user_data_by_nameid");

    check_is_admin(&claims)?;

    let user_data = sqlx::query_as::<_, UserData>(
        "SELECT
            username,
            email,
            name,
            surname
         FROM users_data
         WHERE username = ?"
    )
        .bind(name_id)
        .fetch_one(&state.db)
        .await
        .map_err(|e|{(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())})?;

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
        .fetch_one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

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
            users.id,
            users.username,
            users.password_hash,
            users.permission,
            CASE WHEN users.valid = 'true' THEN 1 ELSE 0 END AS valid,
            users_data.name,
            users_data.surname,
            users_data.email
         FROM users
         LEFT JOIN users_data ON users.id = users_data.username
         WHERE users.username = ?"
    )
        .bind(username)
        .fetch_optional(pool)
        .await?;

    Ok(user)
}
