use crate::auth::claims::Claims;
use crate::user::{match_role, User, UserRola};
use http::StatusCode;
use sqlx::SqlitePool;


pub fn check_is_admin(claims: &Claims) -> Result<(), (StatusCode, String)>{

    let rola = match_role(&claims.role);
    if rola != UserRola::Admin {
        return Err((
            StatusCode::FORBIDDEN,
            "Brak uprawnień. Ta operacja wymaga roli Admin.".to_string(),
        ));
    }
    Ok(())
}
pub fn check_is_user(claims: &Claims) -> Result<(), (StatusCode, String)>{

    let rola = match_role(&claims.role);
    if !matches!(rola, UserRola::Admin | UserRola::User) {
        return Err((
            StatusCode::FORBIDDEN,
            "Brak uprawnień. Ta operacja wymaga roli Admin|User.".to_string(),
        ));
    }
    Ok(())
}
pub async fn check_is_valid(claims: &Claims, pool: &SqlitePool) -> Result<(), (StatusCode, String)>{

    let id = &claims.sub;
    let user_valid: bool = sqlx::query_scalar::<_, bool>("SELECT valid FROM users WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            eprintln!("Błąd bazy danych przy sprawdzaniu statusu użytkownika: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Błąd wewnętrzny serwera.".to_string(),
            )
        })?;

    if !user_valid {
        return Err((
            StatusCode::FORBIDDEN,
            "Twoje konto nie jest aktywne lub zweryfikowane.".to_string(),
        ));
    }

    Ok(())
}

pub fn check_is_own_acc(claims: &Claims, payload: &User) -> Result<(), (StatusCode, String)>{
    let claim_id = claims.sub;
    let user_id = payload.id;

    if claim_id != user_id {
        return Err((
            StatusCode::FORBIDDEN,
            "Błędny użytkownik (mismatch id).".to_string(),
        ));
    }

    Ok(())
}
