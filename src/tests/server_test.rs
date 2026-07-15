


#[cfg(test)]
mod tests {

    use axum::{body::Body, http::{Request, StatusCode}};
    use tower::ServiceExt;
    #[tokio::test]
    async fn test_root_path() {
        let app = app();

        // Tworzysz żądanie
        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
