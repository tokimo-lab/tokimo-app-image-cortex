use std::sync::Arc;

use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::db::repos::system_config_repo::SystemConfigSection;
use crate::error::AppError;
use crate::services;
use crate::state::AppState;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS)]
#[ts(export)]
pub enum AnalysisType {
    Ocr,
    Face,
    Clip,
    Gps,
    All,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS)]
#[ts(export)]
pub struct AnalyzeRequest {
    pub path: String,
    pub analysis_type: AnalysisType,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS)]
#[ts(export)]
pub struct AnalyzeResponse {
    pub path: String,
    pub ocr: Option<services::ocr::OcrResult>,
    pub face: Option<services::face::FaceResult>,
    pub clip: Option<services::clip::ClipResult>,
    pub gps: Option<services::geo::GpsResult>,
}

pub async fn analyze(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AnalyzeRequest>,
) -> Result<Json<AnalyzeResponse>, AppError> {
    let image_bytes = services::image_loader::load_image_bytes(&state.http_client, &req.path).await?;

    let geo_settings: crate::config::GeoSettings = {
        use crate::db::repos::system_config_repo::SystemConfigRepo;
        SystemConfigRepo::get(&state.db)
            .await
            .unwrap_or_else(|_| crate::config::GeoSettings::default_value())
    };

    let (ocr, face, clip, gps) = match req.analysis_type {
        AnalysisType::Ocr => {
            let ocr = services::ocr::analyze(&state.ai_worker, image_bytes).await?;
            (Some(ocr), None, None, None)
        }
        AnalysisType::Face => {
            let face = services::face::analyze(&state.ai_worker, image_bytes).await?;
            (None, Some(face), None, None)
        }
        AnalysisType::Clip => {
            let clip = services::clip::analyze(&state.ai_worker, image_bytes).await?;
            (None, None, Some(clip), None)
        }
        AnalysisType::Gps => {
            let gps = services::geo::analyze(&state.http_client, &image_bytes, &geo_settings).await?;
            (None, None, None, Some(gps))
        }
        AnalysisType::All => {
            let (ocr_r, face_r, clip_r, gps_r) = tokio::join!(
                services::ocr::analyze(&state.ai_worker, image_bytes.clone()),
                services::face::analyze(&state.ai_worker, image_bytes.clone()),
                services::clip::analyze(&state.ai_worker, image_bytes.clone()),
                services::geo::analyze(&state.http_client, &image_bytes, &geo_settings),
            );
            (ocr_r.ok(), face_r.ok(), clip_r.ok(), gps_r.ok())
        }
    };

    Ok(Json(AnalyzeResponse {
        path: req.path,
        ocr,
        face,
        clip,
        gps,
    }))
}
