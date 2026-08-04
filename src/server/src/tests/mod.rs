#[cfg(test)]
mod tests {
    use crate::{build_router, load_env_data};
    use axum::{body::Body, http::{Request, StatusCode}};
    use tower::ServiceExt;
    use sqlite_serv::AppState;


    #[tokio::test]
    async fn test_mojego_routera() {

        load_env_data().ok();
        let pool = sqlx::SqlitePool::connect_lazy("sqlite::memory:").unwrap();
        sqlx::migrate!("../../migrations/data").run(&pool).await.unwrap();

        let (ws_broadcast_tx, _) = tokio::sync::broadcast::channel(16);
        let state = AppState { db: pool, ws_broadcast_tx };

        let app = build_router(state);

        // tworzy request
        let request = Request::builder()
            .uri("/")
            .body(Body::empty())
            .unwrap();

        // wstrzykiwanie ConnectInfo bezpośrednio do requestu, żeby Governor mógł odczytać IP
        let request = tower::ServiceBuilder::new()
            .layer(axum::extract::Extension(axum::extract::ConnectInfo(
                std::net::SocketAddr::from(([127, 0, 0, 1], 8080))
            )))
            .service(tower::service_fn(|req| async move { Ok::<_, std::convert::Infallible>(req) }))
            .oneshot(request)
            .await
            .unwrap();

        // wysyłam przygotowany request do routera
        let response = app.oneshot(request).await.unwrap();

        // Debugowanie jeśli coś dalej nie tak
        if response.status().is_server_error() {
            let (parts, body) = response.into_parts();
            let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
            panic!("Status: {:?}, Error: {:?}", parts.status, String::from_utf8_lossy(&bytes));
        }

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_ws_handshake_dziala() {
        load_env_data().ok();
        // let _ = JWT_SECRET.set("test_secret".as_bytes().to_vec());
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

        // klient, który obsługuje WebSocket upgrade
        // W testach `tokio_tungstenite`
        let url = format!("ws://{}/wss", addr);
        let (ws_stream, response) = tokio_tungstenite::connect_async(url)
            .await
            .expect("Nie udało się połączyć z WebSocketem");

        // asercja: 101 Switching Protocols
        assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);
    }

    // #[tokio::test(flavor = "current_thread")]
    // async fn test_wszystkie_funkcjonalnosci_systemu() {
    //     println!("poczatek");
    //     // let manifest_dir = env!("CARGO_MANIFEST_DIR");
    //     let cert_path = format!("../../cert.pem");
    //     let key_path = format!("../../key.pem");
    //     let config = RustlsConfig::from_pem_file(cert_path,key_path).await.unwrap();
    //     struct DaneDoUsera{
    //         login: String,
    //         haslo: String,
    //     }
    //
    //     let dane_usera = DaneDoUsera{
    //         login: "admin_jan".to_string(),
    //         haslo: "tajne_haslo_123".to_string(),
    //     };
    //
    //     // Setup bazy i serwera
    //     let _ = JWT_SECRET.set("test_secret_key_12345".as_bytes().to_vec());
    //     let _ = PEPPER_KEY.set("test_pepper_key_abc_123".to_string());
    //     let pool = sqlx::SqlitePool::connect_lazy("sqlite::memory:").unwrap();
    //     sqlx::migrate!("../../migrations/data").run(&pool).await.unwrap();
    //
    //     let user = User::new(
    //         &dane_usera.login,
    //         None,
    //         None,
    //         None,
    //         &dane_usera.haslo
    //     ).expect("Błąd tworzenia użytkownika w teście");
    //
    //     sqlx::query!(
    //         "INSERT INTO users (username, password_hash, permission, valid) VALUES (?, ?, ?, ?)",
    //             user.username,
    //             user.password_hash,
    //             format!("{:?}", user.permission),
    //             user.valid
    //         )
    //         .execute(&pool)
    //         .await
    //         .unwrap();
    //
    //     let (ws_broadcast_tx, _) = tokio::sync::broadcast::channel(16);
    //     let state = AppState { db: pool, ws_broadcast_tx };
    //     let app = build_router(state.clone());
    //
    //     let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    //     let addr = listener.local_addr().unwrap();
    //     println!("przed spawnem");
    //     tokio::spawn(async move {
    //         axum_server::bind_rustls(addr, config)
    //             .serve(app.into_make_service_with_connect_info::<SocketAddr>())
    //             .await
    //             .unwrap();
    //     });
    //
    //     let client = reqwest::Client::new();
    //     let base_url = format!("https://{}", addr);
    //
    //     // TEST: Logowanie
    //     let login_res = match client.post(format!("{}/usr/login", base_url))
    //         .json(&serde_json::json!({"username": &dane_usera.login, "password": &dane_usera.haslo}))
    //         .send().await
    //     {
    //         Ok(res) => res,
    //         Err(e)=> panic!("Failed to login: {}",e)
    //     };
    //     assert_eq!(login_res.status(), StatusCode::OK);
    //     let login_data = login_res.json::<serde_json::Value>().await.unwrap();
    //     let token = login_data["token"].as_str().unwrap();
    //
    //     // TEST: Pobieranie produktów (API)
    //     let prod_res = client.get(format!("{}/api/products", base_url)).send().await.unwrap();
    //     assert_eq!(prod_res.status(), StatusCode::OK);
    //
    //     // TEST: Pobieranie modeli (API)
    //     let mod_res = client.get(format!("{}/api/models", base_url)).send().await.unwrap();
    //     assert_eq!(mod_res.status(), StatusCode::OK);
    //
    //     // TEST: WebSocket (Handshake + Dane startowe)
    //     let ws_url = format!("ws://{}/wss", addr);
    //     let request = http::Request::builder()
    //         .uri(ws_url)
    //         .header("Host", addr.to_string()) // DODAJ TĘ LINIĘ
    //         .header("Upgrade", "websocket")
    //         .header("Connection", "Upgrade")
    //         .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
    //         .header("Sec-WebSocket-Version", "13")
    //         .header("Authorization", format!("Bearer {}", token))
    //         .body(())
    //         .unwrap();
    //
    //     let (ws_stream, _) = tokio_tungstenite::connect_async(request).await.unwrap();
    //     let (_, mut receiver) = ws_stream.split();
    //
    //     // Odbierz produkty wysłane przez WebSocket jako "dane startowe"
    //     let initial_msg = receiver.next().await.unwrap().unwrap().to_text().unwrap().to_string();
    //     let produkty_ws: serde_json::Value = serde_json::from_str(&initial_msg).unwrap();
    //     assert!(produkty_ws.is_array(), "WebSocket nie wysłał poprawnej listy produktów");
    //
    //     // TEST: Wysłanie broadcastu przez serwer i odebranie przez WS
    //     let update_msg = "test_update";
    //     state.ws_broadcast_tx.send(update_msg.to_string()).unwrap();
    //
    //     let broadcast_msg = receiver.next().await.unwrap().unwrap().to_text().unwrap().to_string();
    //     assert_eq!(broadcast_msg, update_msg);
    //
    //     println!("Wszystkie testy końcowe przeszły pomyślnie!");
    // }
}