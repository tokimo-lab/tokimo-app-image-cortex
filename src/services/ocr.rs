use serde::Serialize;
use tokimo_perception::worker::client::AiWorkerClient;
use ts_rs::TS;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS)]
#[ts(export)]
pub struct OcrResult {
    pub items: Vec<OcrItem>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS)]
#[ts(export)]
pub struct OcrItem {
    pub text: String,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub w: Option<f64>,
    pub h: Option<f64>,
    pub angle: f64,
    pub score: Option<f64>,
    pub paragraph_id: i32,
    pub corners: Option<Vec<[f64; 2]>>,
}

pub async fn analyze(ai: &AiWorkerClient, image_bytes: Vec<u8>) -> Result<OcrResult, AppError> {
    let items = ai
        .ocr(image_bytes, None, None)
        .await
        .map_err(|e| AppError::Internal(format!("OCR failed: {e}")))?;

    let coord = |v: f32| -> Option<f64> { if v < 0.0 { None } else { Some(f64::from(v)) } };

    let ocr_items = items
        .into_iter()
        .filter(|item| !item.text.trim().is_empty())
        .map(|item| OcrItem {
            text: item.text,
            x: coord(item.x),
            y: coord(item.y),
            w: coord(item.w),
            h: coord(item.h),
            angle: f64::from(item.angle),
            score: Some(f64::from(item.score)),
            paragraph_id: item.paragraph_id as i32,
            corners: item
                .corners
                .map(|c| c.iter().map(|(x, y)| [f64::from(*x), f64::from(*y)]).collect()),
        })
        .collect();

    Ok(OcrResult { items: ocr_items })
}
