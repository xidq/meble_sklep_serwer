// use server::claims::claims::claims_match;
// use auth::file_handling::json_handling::file_serialisation;
// use axum::response::IntoResponse;
// use axum::Json;
// use http::{HeaderMap, StatusCode};
// use id_handling::create::get_new_id;
// use id_handling::enums_structs::ModelData;
// use std::fs::File;
// use std::io::BufReader;
// use std::path::Path;
//
// const MODEL_PATH: &str = "src/api/models/models.json";
//
// pub async fn model_data_storage(
//     headers: HeaderMap,
//     Json(new_model): Json<ModelData>,
// ) -> impl IntoResponse {
//
//     let claims = match claims_match(&headers) {
//         Ok(c) => c,
//         Err(error_response) => return error_response, // Zwraca 401, 400 lub 403
//     };
//
//     if claims.role != "Admin" {
//         return (StatusCode::FORBIDDEN, "Brak uprawnień administratora. Wymagana rola: Admin").into_response();
//     }
//
//     println!("Zgoda na edycję przyznana dla admina: {}", claims.username);
//
//     let mut models = load_models();
//
//
//     match get_new_id(new_model, &mut models) {
//         Ok(_) => {  }
//         Err(error_response) => {
//             return error_response.into_response();
//         }
//     }
//
//     file_serialisation(MODEL_PATH, models)
// }
//
// fn load_models() -> Vec<ModelData> {
//     let path = Path::new(MODEL_PATH);
//     if let Ok(file) = File::open(path) {
//         let reader = BufReader::new(file);
//         serde_json::from_reader(reader).unwrap_or_else(|_| vec![])
//     } else {
//         vec![]
//     }
// }
//
// pub async fn list_models() -> (StatusCode, Json<Vec<ModelData>>)  {
//     let models = load_models();
//     (StatusCode::OK, Json(models))
// }
