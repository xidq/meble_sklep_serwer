use crate::auth::claims::Claims;
use crate::auth::permissions::check_is_admin;
use crate::user::{pepper_password, PasswordChange, User, UserData};
use crate::AppState;
use axum::extract::State;
use axum::Json;
use bcrypt::verify;
use http::StatusCode;

pub async fn handle_admin_edit_user_valid(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<User>,
) -> Result<StatusCode, (StatusCode, String)> {
    println!("Odebrano żądanie zmiany produktu id: {}", payload.id);
    check_is_admin(&claims)?;
    
    sqlx::query(
        r#"
            UPDATE users
            SET
                valid = COALESCE(?, valid)
                WHERE id = ?
        "#
    )
        .bind(payload.valid)
        .bind(payload.id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::ACCEPTED)
}
pub async fn handle_admin_edit_user_data(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<UserData>,
) -> Result<StatusCode, (StatusCode, String)> {
    println!("Odebrano żądanie zmiany produktu name_id: {}", payload.username);
    check_is_admin(&claims)?;

    sqlx::query(
        r#"
            UPDATE users_data
            SET
                email = COALESCE(?, email),
                name = COALESCE(?, name),
                surname = COALESCE(?, surname)
                WHERE username = ?
        "#
    )
        .bind(&payload.email)
        .bind(&payload.name)
        .bind(&payload.surname)
        .bind(&payload.username)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::ACCEPTED)
}
pub async fn handler_edit_user_profile(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<UserData>,
) -> Result<StatusCode, (StatusCode, String)> {

    sqlx::query(
        r#"
        UPDATE users_data
        SET
            email = COALESCE(?, email),
            name = COALESCE(?, name),
            surname = COALESCE(?, surname)
        WHERE username = ?
        "#
    )
        .bind(&payload.email)
        .bind(&payload.name)
        .bind(&payload.surname)
        .bind(&claims.username)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::ACCEPTED)
}
pub async fn handler_edit_user_password(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<PasswordChange>,
) -> Result<StatusCode, (StatusCode, String)> {

    let existing_pass: String = sqlx::query_scalar(
        r#"
            SELECT password_hash FROM users where id = ?
            "#
    )
        .bind(claims.sub)
        .fetch_one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let peppered = pepper_password(&payload.old_password);
    let conf_pass = verify(&peppered, &existing_pass).unwrap_or(false);
    match conf_pass{
        true => {
            let peppered_new = pepper_password(&payload.new_password);
        sqlx::query(
        r#"
            UPDATE users
            SET
                password_hash = ?
            WHERE id = ?
        "#
            )
                .bind(&peppered_new)
                .bind(claims.sub)
                .execute(&state.db)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            Ok(StatusCode::OK)
        }
        false => {Err((StatusCode::UNAUTHORIZED, "Nieprawidłowe stare hasło".to_string()))}
    }

}
// pub async fn handle_edit_user_by_user(
//     State(state): State<AppState>,
//     claims: Claims,
//     Json(mut payload): Json<User>,
// ) -> Result<StatusCode, (StatusCode, String)> {
//     println!("Odebrano żądanie zmiany produktu id: {}", payload.id);
//
//     check_is_own_acc(&claims, &payload)?;
//     // if claims.sub != payload.id {
//     //     return Err((
//     //         StatusCode::FORBIDDEN,
//     //         "Brak uprawnień. Ta operacja wymaga odpowiedniego użytkownika.".to_string(),
//     //     ));
//     // }
//
//     payload.permission = match_role(&claims.role);
//     edit_user(
//         &state.db,
//         &payload,
//     )
//         .await
//         .map_err(|e| match e {
//             sqlx::Error::RowNotFound => (StatusCode::NOT_FOUND, "Product not found".to_string()),
//             _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
//         })?;
//
//     Ok(StatusCode::NO_CONTENT)
// }
// pub async fn edit_user(pool: &SqlitePool, user: &User) -> Result<(), sqlx::Error> {
//     // let pepper = PEPPER_KEY.get().expect("PEPPER_KEY nie jest zainicjalizowany");
//     if user.password_hash.len() < 8 || user.password_hash.len() > 100 {
//         return Err(sqlx::Error::Protocol(
//             "Hasło musi mieć od 8 do 100 znaków".to_string()
//         ));
//     }
//     let peppered = pepper_password(&user.password_hash);
//     // DEFAULT_COST - 12 - optymalna siła hash, anty brute-force, MAX_COST - 31, MIN_COST - 4
//     let password_hash = hash(&peppered, 12).map_err(|e| {
//         sqlx::Error::Protocol(format!("Błąd hashowania: {}", e))
//     })?;
//     sqlx::query(
//         r#"
//         UPDATE users
//         SET
//             name = ?,
//             email = ?,
//             password_hash = ?,
//             permission = ?
//         WHERE id = ?
//         "#
//     )
//         .bind(&user.name)
//         .bind(&user.email)
//         .bind(&password_hash)
//         .bind(&user.permission)
//         .bind(user.id) // To ID z lokacji WHERE
//         .execute(pool)
//         .await?;
//
//     Ok(())
// }