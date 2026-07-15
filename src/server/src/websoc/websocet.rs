use crate::response::login::LoginResponse;
use crate::requests;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use chrono::{Duration, Utc};
use futures_util::{SinkExt, StreamExt};
use jsonwebtoken::{encode, EncodingKey, Header};
use sqlite_serv::auth::claims::Claims;
use sqlite_serv::auth::jwt::JWT_SECRET;
use sqlite_serv::product::get::get_products_list;
use sqlite_serv::AppState;
use sqlite_serv::user::get::get_user_by_username;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    // claims: Claims,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}


/// Autorisation handle
pub async fn login_handler(
    State(state): State<AppState>,
    Json(payload): Json<requests::login::LoginRequest>
) -> impl IntoResponse {

    let found_user = match get_user_by_username(&state.db, &payload.username).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({ "error": "Użytkownik nie istnieje" }))).into_response();
        }
        Err(e) => {
            eprintln!("Błąd bazy danych podczas logowania: {:?}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Błąd serwera").into_response();
        }
    };

    println!("login_handler found username {:?}", found_user);

    if found_user.verify_password(&payload.password) {
        // Token będzie ważny przez 24 godziny
        let expiration = Utc::now()
            .checked_add_signed(Duration::try_hours(24).unwrap().abs())
            .expect("Wadliwy timestamp")
            .timestamp();

        let claims = Claims {
            sub: found_user.id,
            username: found_user.username.clone(),
            role: format!("{:?}", found_user.permission),
            exp: expiration,
        };

        match encode(&Header::default(), &claims, &EncodingKey::from_secret(JWT_SECRET.get().expect("Klucz JWT nie został zainicjalizowany!"))) {
            Ok(token) => (
                StatusCode::OK,
                Json(LoginResponse {
                    token,
                    username: found_user.username.clone(),
                    role: format!("{:?}", found_user.permission),
                })
            ).into_response(),
            Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Błąd generowania sesji").into_response(),
        }
    } else {
        (StatusCode::UNAUTHORIZED, Json(serde_json::json!({ "error": "Niepoprawne hasło" }))).into_response()
    }
}

/// WebSocket handler
pub async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    println!("Nowy klient połączył się przez WebSocket");
    
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                println!("Klient napisał: {}", text);
            }
        }
        println!("Klient rozłączył się (nasłuchiwanie przerwane).");
    });

    let mut send_task = tokio::spawn(async move {
        let mut rx = state.ws_broadcast_tx.subscribe();

        // DANE STARTOWE (Wysyłamy tylko RAZ), zakładka admin
        if let Ok(produkty) = get_products_list(&state.db).await {
            let json = serde_json::to_string(&produkty).unwrap_or_else(|_| "[]".to_string());

            if let Err(e) = sender.send(Message::Text(json.into())).await {
                eprintln!("BŁĄD wysyłki początkowej: {}", e);
                return; // Klient się rozłączył zanim zdążyliśmy wysłać, kończymy task
            }
        }

        // 2. PĘTLA BROADCAST
        while let Ok(msg) = rx.recv().await {
            if let Err(e) = sender.send(Message::Text(msg.into())).await {
                eprintln!("BŁĄD wysyłki broadcast: {}", e);
                break; // Kończymy pętlę w przypadku błędu wysyłki
            }
        }
    });

    // Pilnowanie zakończenia zadań – jeśli jedno padnie/zakończy się, ubijamy drugie
    tokio::select! {
        _ = (&mut recv_task) => send_task.abort(),
        _ = (&mut send_task) => recv_task.abort(),
    };

    println!("Czyszczenie zasobów dla rozłączonego klienta zakończone.");
}