use axum::extract::ws::{Message, WebSocket};
use axum::Json;
use axum::response::IntoResponse;
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use axum::http::StatusCode;
use axum::extract::{State, WebSocketUpgrade};
use futures_util::{SinkExt, StreamExt};
use auth::claims_thingy::claims::Claims;
use auth::jwt::{get_user_by_username, JWT_SECRET};
use sqlite_serv::sql::AppState;
use crate::{requests};
use crate::response::login::LoginResponse;
use id_handling::enums_structs::Product;


pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

pub async fn login_handler(
    State(state): State<AppState>, // <-- Dodajemy dostęp do stanu bazy danych
    Json(payload): Json<requests::login::LoginRequest>
) -> impl IntoResponse {

    println!("Próba pobrania produktów z bazy...");
    let produkty = pobierz_produkty_jako_json(&state.db).await;
    println!("Wynik pobrania: {:?}", produkty);

    // Szukamy użytkownika bezpośrednio w bazie za pomocą nowej funkcji
    let found_user = match get_user_by_username(&state.db_usr, &payload.username).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({ "error": "Użytkownik nie istnieje" }))).into_response();
        }
        Err(e) => {
            eprintln!("Błąd bazy danych podczas logowania: {:?}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Błąd serwera").into_response();
        }
    };

    // Weryfikacja hasła (wykorzystuje dokładnie tę samą metodę struktury User, co wcześniej)
    if found_user.verify_password(&payload.password) {
        // Token będzie ważny przez 24 godziny
        let expiration = Utc::now()
            .checked_add_signed(Duration::try_hours(24).unwrap().abs())
            .expect("Wadliwy timestamp")
            .timestamp();

        let claims = Claims {
            sub: found_user.id,
            username: found_user.name.clone(),
            role: format!("{:?}", found_user.permission),
            exp: expiration,
        };

        match encode(&Header::default(), &claims, &EncodingKey::from_secret(JWT_SECRET.get().expect("Klucz JWT nie został zainicjalizowany!"))) {
            Ok(token) => (
                StatusCode::OK,
                Json(LoginResponse {
                    token,
                    username: found_user.name.clone(),
                    role: format!("{:?}", found_user.permission),
                })
            ).into_response(),
            Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Błąd generowania sesji").into_response(),
        }
    } else {
        (StatusCode::UNAUTHORIZED, Json(serde_json::json!({ "error": "Niepoprawne hasło" }))).into_response()
    }
}

pub async fn handle_socket(socket: WebSocket, state: AppState) -> () {
    let (mut sender, mut receiver) = socket.split();

    println!("Nowy klient połączył się przez WebSocket");

    // --- WYSYŁANE DANE NA "DZIEŃ DOBRY" ---
    // Gdy klient (np. tablet) się połączy, od razu ładujemy mu aktualną listę
    if let Ok(aktualne_produkty) = pobierz_produkty_jako_json(&state.db).await {
        let _ = sender.send(Message::Text(aktualne_produkty.into())).await;
    }

    // Zadanie odbierające (bez zmian - loguje wiadomości od klienta)
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

        // 1. DANE STARTOWE
        if let Ok(json) = pobierz_produkty_jako_json(&state.db).await {
            if let Err(e) = sender.send(Message::Text(json.into())).await {
                eprintln!("BŁĄD wysyłki początkowej: {}", e); // <--- ZOBACZ TO W LOGACH!
                return;
            }
        }

        // 2. PĘTLA BROADCAST
        while let Ok(msg) = rx.recv().await {
            if let Err(e) = sender.send(Message::Text(msg.into())).await {
                eprintln!("BŁĄD wysyłki broadcast: {}", e); // <--- ZOBACZ TO W LOGACH!
                break;
            }
        }
    });

    // Pilnowanie zakończenia zadań
    tokio::select! {
        _ = (&mut recv_task) => send_task.abort(),
        _ = (&mut send_task) => recv_task.abort(),
    };

    println!("Czyszczenie zasobów dla rozłączonego klienta zakończone.");
}

async fn pobierz_produkty_jako_json(pool: &sqlx::SqlitePool) -> Result<String, sqlx::Error> {
    let produkty: Vec<Product> = sqlx::query_as::<_, Product>(
        "SELECT id, name, ... FROM products"
    )
        .fetch_all(pool)
        .await?;

    let json = serde_json::to_string(&produkty).unwrap_or_else(|_| "[]".to_string());
    Ok(json)
}