use std::sync::Arc;

use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::config::{AiSettings, GeoSettings};
use crate::db::repos::system_config_repo::SystemConfigRepo;
use crate::error::AppError;
use crate::state::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS)]
#[ts(export)]
pub struct GeoSettingsOutput {
    pub provider: String,
    pub amap_api_key: Option<String>,
    pub amap_secret: Option<String>,
    pub qqmap_api_key: Option<String>,
    pub qqmap_secret_key: Option<String>,
    pub tianditu_server_key: Option<String>,
    pub mapbox_access_token: Option<String>,
    pub maptiler_api_key: Option<String>,
}

impl From<GeoSettings> for GeoSettingsOutput {
    fn from(s: GeoSettings) -> Self {
        Self {
            provider: s.provider,
            amap_api_key: s.amap_api_key,
            amap_secret: s.amap_secret,
            qqmap_api_key: s.qqmap_api_key,
            qqmap_secret_key: s.qqmap_secret_key,
            tianditu_server_key: s.tianditu_server_key,
            mapbox_access_token: s.mapbox_access_token,
            maptiler_api_key: s.maptiler_api_key,
        }
    }
}

pub async fn get_geo(State(state): State<Arc<AppState>>) -> Result<Json<GeoSettingsOutput>, AppError> {
    let settings: GeoSettings = SystemConfigRepo::get(&state.db).await?;
    Ok(Json(settings.into()))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateGeoRequest {
    pub provider: Option<String>,
    pub amap_api_key: Option<String>,
    pub amap_secret: Option<String>,
    pub qqmap_api_key: Option<String>,
    pub qqmap_secret_key: Option<String>,
    pub tianditu_server_key: Option<String>,
    pub mapbox_access_token: Option<String>,
    pub maptiler_api_key: Option<String>,
}

pub async fn update_geo(
    State(state): State<Arc<AppState>>,
    Json(req): Json<UpdateGeoRequest>,
) -> Result<Json<GeoSettingsOutput>, AppError> {
    let current: GeoSettings = SystemConfigRepo::get(&state.db).await?;
    let updated = GeoSettings {
        provider: req.provider.unwrap_or(current.provider),
        amap_api_key: req.amap_api_key.or(current.amap_api_key),
        amap_secret: req.amap_secret.or(current.amap_secret),
        qqmap_api_key: req.qqmap_api_key.or(current.qqmap_api_key),
        qqmap_secret_key: req.qqmap_secret_key.or(current.qqmap_secret_key),
        tianditu_server_key: req.tianditu_server_key.or(current.tianditu_server_key),
        mapbox_access_token: req.mapbox_access_token.or(current.mapbox_access_token),
        maptiler_api_key: req.maptiler_api_key.or(current.maptiler_api_key),
    };
    SystemConfigRepo::set(&state.db, &updated).await?;
    Ok(Json(updated.into()))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS)]
#[ts(export)]
pub struct AiSettingsOutput {
    pub ocr_model_name: String,
    pub ocr_aux_model_name: Option<String>,
}

impl From<AiSettings> for AiSettingsOutput {
    fn from(s: AiSettings) -> Self {
        Self {
            ocr_model_name: s.ocr_model_name,
            ocr_aux_model_name: s.ocr_aux_model_name,
        }
    }
}

pub async fn get_ai(State(state): State<Arc<AppState>>) -> Result<Json<AiSettingsOutput>, AppError> {
    let settings: AiSettings = SystemConfigRepo::get(&state.db).await?;
    Ok(Json(settings.into()))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAiRequest {
    pub ocr_model_name: Option<String>,
    pub ocr_aux_model_name: Option<String>,
}

pub async fn update_ai(
    State(state): State<Arc<AppState>>,
    Json(req): Json<UpdateAiRequest>,
) -> Result<Json<AiSettingsOutput>, AppError> {
    let current: AiSettings = SystemConfigRepo::get(&state.db).await?;
    let updated = AiSettings {
        ocr_model_name: req.ocr_model_name.unwrap_or(current.ocr_model_name),
        ocr_aux_model_name: req.ocr_aux_model_name.or(current.ocr_aux_model_name),
    };
    SystemConfigRepo::set(&state.db, &updated).await?;
    Ok(Json(updated.into()))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestGeoRequest {
    #[allow(dead_code)]
    pub lat: f64,
    #[allow(dead_code)]
    pub lon: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS)]
#[ts(export)]
pub struct TestGeoResponse {
    pub success: bool,
    pub province: Option<String>,
    pub city: Option<String>,
    pub district: Option<String>,
    pub formatted_address: Option<String>,
    pub error: Option<String>,
}

pub async fn test_geo(
    State(state): State<Arc<AppState>>,
    Json(_req): Json<TestGeoRequest>,
) -> Result<Json<TestGeoResponse>, AppError> {
    let settings: GeoSettings = SystemConfigRepo::get(&state.db).await?;

    match crate::services::geo::analyze(&state.http_client, &[], &settings).await {
        Ok(result) => Ok(Json(TestGeoResponse {
            success: true,
            province: result.province,
            city: result.city,
            district: result.district,
            formatted_address: result.formatted_address,
            error: None,
        })),
        Err(e) => Ok(Json(TestGeoResponse {
            success: false,
            province: None,
            city: None,
            district: None,
            formatted_address: None,
            error: Some(e.to_string()),
        })),
    }
}
