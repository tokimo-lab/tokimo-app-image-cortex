use serde::Serialize;
use tokimo_perception::worker::client::AiWorkerClient;
use ts_rs::TS;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS)]
#[ts(export)]
pub struct FaceResult {
    pub faces: Vec<FaceItem>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS)]
#[ts(export)]
pub struct FaceItem {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub confidence: f32,
    pub embedding: Vec<f32>,
}

pub async fn analyze(ai: &AiWorkerClient, image_bytes: Vec<u8>) -> Result<FaceResult, AppError> {
    let detections = ai
        .detect_faces(image_bytes, None)
        .await
        .map_err(|e| AppError::Internal(format!("Face detection failed: {e}")))?;

    let faces = detections
        .into_iter()
        .map(|d| FaceItem {
            x: d.x,
            y: d.y,
            w: d.w,
            h: d.h,
            confidence: d.confidence,
            embedding: d.embedding,
        })
        .collect();

    Ok(FaceResult { faces })
}
