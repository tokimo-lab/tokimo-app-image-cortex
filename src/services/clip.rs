use serde::Serialize;
use tokimo_perception::worker::client::AiWorkerClient;
use ts_rs::TS;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS)]
#[ts(export)]
pub struct ClipResult {
    pub embedding: Vec<f32>,
}

pub async fn analyze(ai: &AiWorkerClient, image_bytes: Vec<u8>) -> Result<ClipResult, AppError> {
    let embedding = ai
        .clip_image(image_bytes, None)
        .await
        .map_err(|e| AppError::Internal(format!("CLIP failed: {e}")))?;

    if embedding.len() != 512 {
        return Err(AppError::Internal(format!(
            "CLIP returned {} dims, expected 512",
            embedding.len()
        )));
    }

    Ok(ClipResult { embedding })
}
