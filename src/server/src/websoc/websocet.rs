use axum::extract::ws::{Message, WebSocket};
use axum::Json;
use axum::response::IntoResponse;
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use axum::http::StatusCode;
use axum::extract::WebSocketUpgrade;
use futures_util::{SinkExt, StreamExt};
use auth::claims_thingy::claims::Claims;
use auth::jwt::JWT_SECRET;
use crate::{requests};
use crate::response::login::LoginResponse;

pub async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

pub async fn login_handler(Json(payload): Json<requests::login::LoginRequest>) -> impl IntoResponse {
    
    let users = auth::jwt::load_users_from_json();
    let found_user = users.iter().find(|u| u.name == payload.username);

    if let Some(user) = found_user {
        if user.verify_password(&payload.password) {
            // Token będzie ważny przez 24 godziny
            let expiration = Utc::now()
                .checked_add_signed(Duration::try_hours(24).unwrap().abs())
                .expect("Wadliwy timestamp")
                .timestamp();

            let claims = Claims {
                sub: user.id,
                username: user.name.clone(),
                role: format!("{:?}", user.permission),
                exp: expiration,
            };

            match encode(&Header::default(), &claims, &EncodingKey::from_secret(JWT_SECRET.get().expect("Klucz JWT nie został zainicjalizowany!"))) {
                Ok(token) => (
                    StatusCode::OK,
                    Json(LoginResponse {
                        token,
                        username: user.name.clone(),
                        role: format!("{:?}", user.permission),
                    })
                ).into_response(),
                Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Błąd generowania sesji").into_response(),
            }
        } else {
            (StatusCode::UNAUTHORIZED, Json(serde_json::json!({ "error": "Niepoprawne hasło" }))).into_response()
        }
    } else {
        (StatusCode::UNAUTHORIZED, Json(serde_json::json!({ "error": "Użytkownik nie istnieje" }))).into_response()
    }
}

async fn handle_socket(socket: WebSocket) -> () {
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
        let mut counter = 0;
        loop {
            counter += 1;

            // ping do frontu
            let notification = serde_json::json!({
                "type": "LIVE_UPDATE",
                "message": format!("Aktualizacja danych rynkowych #{}", counter),
                "timestamp": chrono::Utc::now().to_rfc3339()
            });

            let json_string = serde_json::to_string(&notification).unwrap();

            if sender.send(Message::Text(json_string.into())).await.is_err() {
                // jak klient uciekł (sysyłanie nie powiodło sie)
                break;
            }

            tokio::time::sleep(core::time::Duration::from_secs(3)).await; // Wyślij co 3 sekundy
        }
    });

    // w8-tin' 4 jakieś zadanie aż sie zakończy
    tokio::select! {
        _ = (&mut recv_task) => send_task.abort(),
        _ = (&mut send_task) => recv_task.abort(),
    };
}