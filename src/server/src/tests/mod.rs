#[cfg(test)]
mod tests {
    use axum::{body::Body, http::{Request, StatusCode}};
    use futures_util::StreamExt;
    use tower::ServiceExt;
    use sqlite_serv::auth::jwt::JWT_SECRET;
    use sqlite_serv::PEPPER_KEY;
    use sqlite_serv::user::User;
    use crate::build_router; // <--- Użyj pełnej ścieżki do funkcji z main.rs
    use crate::AppState;     // <--- Zaimportuj AppState

    #[tokio::test]
    async fn test_mojego_routera() {
        let pool = sqlx::SqlitePool::connect_lazy("sqlite::memory:").unwrap();
        sqlx::migrate!("../../migrations/data").run(&pool).await.unwrap();

        let (ws_broadcast_tx, _) = tokio::sync::broadcast::channel(16);
        let state = AppState { db: pool, ws_broadcast_tx };

        let app = build_router(state);

        // 1. Tworzymy request
        let request = Request::builder()
            .uri("/")
            .body(Body::empty())
            .unwrap();

        // 2. Wstrzykujemy ConnectInfo bezpośrednio do requestu, żeby Governor mógł odczytać IP
        let request = tower::ServiceBuilder::new()
            .layer(axum::extract::Extension(axum::extract::ConnectInfo(
                std::net::SocketAddr::from(([127, 0, 0, 1], 8080))
            )))
            .service(tower::service_fn(|req| async move { Ok::<_, std::convert::Infallible>(req) }))
            .oneshot(request)
            .await
            .unwrap();

        // 3. Wysyłamy tak przygotowany request do routera
        let response = app.oneshot(request).await.unwrap();

        // 4. Debugowanie jeśli coś dalej nie tak
        if response.status().is_server_error() {
            let (parts, body) = response.into_parts();
            let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
            panic!("Status: {:?}, Error: {:?}", parts.status, String::from_utf8_lossy(&bytes));
        }

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
    // #[tokio::test]
    // async fn test_ws_handshake() {
    //     let _ = JWT_SECRET.set("testowy_sekret_ktory_musi_byc_dlugi_i_bezpieczny".to_string().into());
    //     // 1. Setup bazy i stanu
    //     let pool = sqlx::SqlitePool::connect_lazy("sqlite::memory:").unwrap();
    //     sqlx::migrate!("../../migrations/data").run(&pool).await.unwrap();
    //     let (ws_broadcast_tx, _) = tokio::sync::broadcast::channel(16);
    //     let state = AppState { db: pool, ws_broadcast_tx };
    //     let app = build_router(state);
    //
    //     // KROK A: Zaloguj się, żeby dostać token (tak jak to robi JS)
    //     let login_request = Request::builder()
    //         .method("POST")
    //         .uri("/usr/login")
    //         .header("Content-Type", "application/json")
    //         .body(Body::from(r#"{"username": "xidq", "password": "$2b$12$dRYB00aQCzYjESEJIjGMg.0NI45alQJkz74CaFV.DeWGQjM9E9UqK"}"#))
    //         .unwrap();
    //
    //     let login_response = app.clone().oneshot(login_request).await.unwrap();
    //     assert_eq!(login_response.status(), StatusCode::OK);
    //
    //     // Wyciągnij token z odpowiedzi (zakładając, że serwer go zwraca)
    //     let body_bytes = axum::body::to_bytes(login_response.into_body(), usize::MAX).await.unwrap();
    //     let login_data: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    //     let token = login_data["token"].as_str().unwrap().to_string(); // To jest Twój prawdziwy token
    //
    //     // KROK B: Użyj tego tokena w teście WebSocket
    //     let request = Request::builder()
    //         .uri("/ws")
    //         .header("Upgrade", "websocket")
    //         .header("Connection", "Upgrade")
    //         .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
    //         .header("Sec-WebSocket-Version", "13")
    //         .header("Authorization", format!("Bearer {}", token))
    //         .body(Body::empty())
    //         .unwrap();
    //
    //     let response = app.oneshot(request).await.unwrap();
    //
    //     // Debug
    //     if response.status() != StatusCode::SWITCHING_PROTOCOLS {
    //         let (parts, body) = response.into_parts();
    //         let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
    //         panic!("Status: {:?}, Error: {:?}", parts.status, String::from_utf8_lossy(&bytes));
    //     }
    //
    //     assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);
    // }

    #[tokio::test]
    async fn test_ws_handshake_dziala() {
        // 1. Setup
        let _ = JWT_SECRET.set("test_secret".as_bytes().to_vec()); // Inicjalizacja sekretu
        let pool = sqlx::SqlitePool::connect_lazy("sqlite::memory:").unwrap();
        sqlx::migrate!("../../migrations/data").run(&pool).await.unwrap();
        let (ws_broadcast_tx, _) = tokio::sync::broadcast::channel(16);
        let state = AppState { db: pool, ws_broadcast_tx };

        let app = build_router(state);

        // 2. Uruchamiamy serwer testowy na wolnym porcie
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::serve(listener, app.into_make_service_with_connect_info::<std::net::SocketAddr>())
                .await
                .unwrap();
        });

        // 3. Używamy klienta, który faktycznie obsługuje WebSocket upgrade
        // W testach najprościej użyć `tokio_tungstenite`
        let url = format!("ws://{}/ws", addr);
        let (ws_stream, response) = tokio_tungstenite::connect_async(url)
            .await
            .expect("Nie udało się połączyć z WebSocketem");

        // 4. Asercja: 101 Switching Protocols
        assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);
    }

    #[tokio::test]
    async fn test_wszystkie_funkcjonalnosci_systemu() {
        // 1. Setup bazy i serwera
        let _ = JWT_SECRET.set("test_secret_key_12345".as_bytes().to_vec());
        let _ = PEPPER_KEY.set("test_pepper_key_abc_123".to_string());
        let pool = sqlx::SqlitePool::connect_lazy("sqlite::memory:").unwrap();
        sqlx::migrate!("../../migrations/data").run(&pool).await.unwrap();

        let user = User::new(
            "admin_jan",
            None,
            None,
            "tajne_haslo_123"
        ).expect("Błąd tworzenia użytkownika w teście");

        sqlx::query!(
            "INSERT INTO users (username, password_hash, permission, valid) VALUES (?, ?, ?, ?)",
        user.username,
        user.password_hash,
        format!("{:?}", user.permission),
        user.valid
            )
            .execute(&pool)
            .await
            .unwrap();

        let (ws_broadcast_tx, _) = tokio::sync::broadcast::channel(16);
        let state = AppState { db: pool, ws_broadcast_tx };
        let app = build_router(state.clone());

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app.into_make_service_with_connect_info::<std::net::SocketAddr>())
                .await.unwrap();
        });

        let client = reqwest::Client::new();
        let base_url = format!("http://{}", addr);

        // 2. TEST: Logowanie
        let login_res = client.post(format!("{}/usr/login", base_url))
            .json(&serde_json::json!({"username": "admin_jan", "password": "tajne_haslo_123"}))
            .send().await.unwrap();
        assert_eq!(login_res.status(), StatusCode::OK);
        let login_data = login_res.json::<serde_json::Value>().await.unwrap();
        let token = login_data["token"].as_str().unwrap();

        // 3. TEST: Pobieranie produktów (API)
        let prod_res = client.get(format!("{}/api/products", base_url)).send().await.unwrap();
        assert_eq!(prod_res.status(), StatusCode::OK);

        // 4. TEST: Pobieranie modeli (API)
        let mod_res = client.get(format!("{}/api/models", base_url)).send().await.unwrap();
        assert_eq!(mod_res.status(), StatusCode::OK);

        // 5. TEST: WebSocket (Handshake + Dane startowe)
        let ws_url = format!("ws://{}/ws", addr);
        let request = http::Request::builder()
            .uri(ws_url)
            .header("Host", addr.to_string()) // DODAJ TĘ LINIĘ
            .header("Upgrade", "websocket")
            .header("Connection", "Upgrade")
            .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
            .header("Sec-WebSocket-Version", "13")
            .header("Authorization", format!("Bearer {}", token))
            .body(())
            .unwrap();

        let (ws_stream, _) = tokio_tungstenite::connect_async(request).await.unwrap();
        let (_, mut receiver) = ws_stream.split();

        // Odbierz produkty wysłane przez WebSocket jako "dane startowe"
        let initial_msg = receiver.next().await.unwrap().unwrap().to_text().unwrap().to_string();
        let produkty_ws: serde_json::Value = serde_json::from_str(&initial_msg).unwrap();
        assert!(produkty_ws.is_array(), "WebSocket nie wysłał poprawnej listy produktów");

        // 6. TEST: Wysłanie broadcastu przez serwer i odebranie przez WS
        let update_msg = "test_update";
        state.ws_broadcast_tx.send(update_msg.to_string()).unwrap();

        let broadcast_msg = receiver.next().await.unwrap().unwrap().to_text().unwrap().to_string();
        assert_eq!(broadcast_msg, update_msg);

        println!("Wszystkie testy końcowe przeszły pomyślnie!");
    }
}