use serde_json::{Value as JsonValue, json};
use std::sync::Arc;
use uuid::Uuid;

use crate::error::AppError;
use crate::services;
use crate::state::AppState;

pub async fn handle(
    state: &Arc<AppState>,
    _job_id: Uuid,
    params: &JsonValue,
) -> Result<Option<JsonValue>, AppError> {
    let path = params
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("missing path".into()))?;
    let analysis_type = params
        .get("analysisType")
        .and_then(|v| v.as_str())
        .unwrap_or("all");

    let image_bytes = services::image_loader::load_image_bytes(&state.http_client, path).await?;

    let geo_settings: crate::config::GeoSettings = {
        use crate::db::repos::system_config_repo::{SystemConfigRepo, SystemConfigSection};
        SystemConfigRepo::get(&state.db)
            .await
            .unwrap_or_else(|_| crate::config::GeoSettings::default_value())
    };

    let (ocr, face, clip, gps) = match analysis_type {
        "ocr" => {
            let ocr = services::ocr::analyze(&state.ai_worker, image_bytes).await?;
            (Some(ocr), None, None, None)
        }
        "face" => {
            let face = services::face::analyze(&state.ai_worker, image_bytes).await?;
            (None, Some(face), None, None)
        }
        "clip" => {
            let clip = services::clip::analyze(&state.ai_worker, image_bytes).await?;
            (None, None, Some(clip), None)
        }
        "gps" => {
            let gps = services::geo::analyze(&state.http_client, &image_bytes, &geo_settings).await?;
            (None, None, None, Some(gps))
        }
        _ => {
            let (ocr_r, face_r, clip_r, gps_r) = tokio::join!(
                services::ocr::analyze(&state.ai_worker, image_bytes.clone()),
                services::face::analyze(&state.ai_worker, image_bytes.clone()),
                services::clip::analyze(&state.ai_worker, image_bytes.clone()),
                services::geo::analyze(&state.http_client, &image_bytes, &geo_settings),
            );
            (ocr_r.ok(), face_r.ok(), clip_r.ok(), gps_r.ok())
        }
    };

    Ok(Some(json!({
        "path": path,
        "ocr": ocr,
        "face": face,
        "clip": clip,
        "gps": gps,
    })))
}
