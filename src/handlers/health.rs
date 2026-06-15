use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use serde::Serialize;
use ts_rs::TS;

use crate::error::AppError;
use crate::state::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS)]
#[ts(export)]
pub struct HealthResponse {
    pub status: String,
    pub ai_worker_ready: bool,
    pub ocr_ready: bool,
    pub face_ready: bool,
    pub clip_ready: bool,
}

pub async fn health(State(state): State<Arc<AppState>>) -> Result<Json<HealthResponse>, AppError> {
    Ok(Json(HealthResponse {
        status: "ok".to_string(),
        ai_worker_ready: state.models_ready(),
        ocr_ready: state.ocr_ready(),
        face_ready: state.face_ready(),
        clip_ready: state.clip_ready(),
    }))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS)]
#[ts(export)]
pub struct CapabilitiesResponse {
    pub version: String,
    pub analysis_types: Vec<String>,
    pub supported_formats: Vec<String>,
}

pub async fn capabilities() -> Json<CapabilitiesResponse> {
    Json(CapabilitiesResponse {
        version: env!("CARGO_PKG_VERSION").to_string(),
        analysis_types: vec![
            "ocr".to_string(),
            "face".to_string(),
            "clip".to_string(),
            "gps".to_string(),
            "all".to_string(),
        ],
        supported_formats: vec![
            "jpg".to_string(),
            "jpeg".to_string(),
            "png".to_string(),
            "webp".to_string(),
            "gif".to_string(),
            "bmp".to_string(),
            "tiff".to_string(),
            "heic".to_string(),
            "heif".to_string(),
            "avif".to_string(),
            "raw".to_string(),
            "cr2".to_string(),
            "cr3".to_string(),
            "nef".to_string(),
            "arw".to_string(),
            "dng".to_string(),
        ],
    })
}
