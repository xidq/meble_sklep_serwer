use crate::sql::AppState;
use crate::user::{match_role, pepper_password, User, UserRola};
use crate::PEPPER_KEY;
use axum::extract::State;
use axum::Json;
use bcrypt::hash;
use http::StatusCode;
use sqlx::SqlitePool;
use crate::auth::claims::Claims;
use crate::auth::permissions::{check_is_admin, check_is_own_acc};

pub async fn handle_edit_user(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<User>,
) -> Result<StatusCode, (StatusCode, String)> {
    println!("Odebrano żądanie zmiany produktu id: {}", &payload.id);
    check_is_admin(&claims)?;
    edit_user(
        &state.db,
        &payload,
    )
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => (StatusCode::NOT_FOUND, "Product not found".to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        })?;

    Ok(StatusCode::NO_CONTENT)
}
pub async fn handle_edit_user_by_user(
    State(state): State<AppState>,
    claims: Claims,
    Json(mut payload): Json<User>,
) -> Result<StatusCode, (StatusCode, String)> {
    println!("Odebrano żądanie zmiany produktu id: {}", &payload.id);

    check_is_own_acc(&claims, &payload)?;
    // if claims.sub != payload.id {
    //     return Err((
    //         StatusCode::FORBIDDEN,
    //         "Brak uprawnień. Ta operacja wymaga odpowiedniego użytkownika.".to_string(),
    //     ));
    // }

    payload.permission = match_role(&claims.role);
    edit_user(
        &state.db,
        &payload,
    )
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => (StatusCode::NOT_FOUND, "Product not found".to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        })?;

    Ok(StatusCode::NO_CONTENT)
}
pub async fn edit_user(pool: &SqlitePool, user: &User) -> Result<(), sqlx::Error> {
    let pepper = PEPPER_KEY.get().expect("PEPPER_KEY nie jest zainicjalizowany");
    if user.password_hash.len() < 8 || user.password_hash.len() > 100 {
        return Err(sqlx::Error::Protocol(
            "Hasło musi mieć od 8 do 100 znaków".to_string()
        ));
    }
    let peppered = pepper_password(&user.password_hash);
    // DEFAULT_COST - 12 - optymalna siła hash, anty brute-force, MAX_COST - 31, MIN_COST - 4
    let password_hash = hash(&peppered, 12).map_err(|e| {
        sqlx::Error::Protocol(format!("Błąd hashowania: {}", e))
    })?;
    sqlx::query(
        r#"
        UPDATE users
        SET
            name = ?,
            email = ?,
            password_hash = ?,
            permission = ?
        WHERE id = ?
        "#
    )
        .bind(&user.name)
        .bind(&user.email)
        .bind(&password_hash)
        .bind(&user.permission)
        .bind(user.id) // To ID z lokacji WHERE
        .execute(pool)
        .await?;

    Ok(())
}