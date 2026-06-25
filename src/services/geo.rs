use reqwest::Client;
use serde::Serialize;
use tokimo_package_client_api::geocoding::GeocodingClient;
use ts_rs::TS;

use crate::config::GeoSettings;
use crate::error::AppError;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS)]
#[ts(export)]
pub struct GpsResult {
    pub latitude: f64,
    pub longitude: f64,
    pub province: Option<String>,
    pub city: Option<String>,
    pub district: Option<String>,
    pub formatted_address: Option<String>,
}

pub async fn analyze(http: &Client, image_bytes: &[u8], settings: &GeoSettings) -> Result<GpsResult, AppError> {
    let (lat, lon) = extract_gps(image_bytes)?;

    let geo_client = GeocodingClient::new(http.clone());
    let geo = match settings.provider.as_str() {
        "amap" => {
            let Some(key) = settings.amap_api_key.as_deref() else {
                return Ok(coordinates_only(lat, lon));
            };
            let secret = settings.amap_secret.as_deref();
            geo_client
                .amap_reverse_geocode(key, secret, lon, lat)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?
        }
        "qqmap" => {
            let Some(key) = settings.qqmap_api_key.as_deref() else {
                return Ok(coordinates_only(lat, lon));
            };
            let secret = settings.qqmap_secret_key.as_deref();
            geo_client
                .qqmap_reverse_geocode(key, secret, lon, lat)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?
        }
        "tianditu" => {
            let Some(key) = settings.tianditu_server_key.as_deref() else {
                return Ok(coordinates_only(lat, lon));
            };
            geo_client
                .tianditu_reverse_geocode(key, lon, lat)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?
        }
        "mapbox" => {
            let Some(token) = settings.mapbox_access_token.as_deref() else {
                return Ok(coordinates_only(lat, lon));
            };
            geo_client
                .mapbox_reverse_geocode(token, lon, lat)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?
        }
        "maptiler" => {
            let Some(key) = settings.maptiler_api_key.as_deref() else {
                return Ok(coordinates_only(lat, lon));
            };
            geo_client
                .maptiler_reverse_geocode(key, lon, lat)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?
        }
        other => return Err(AppError::Internal(format!("Unknown geo provider: {other}"))),
    };

    Ok(GpsResult {
        latitude: lat,
        longitude: lon,
        province: geo.province,
        city: geo.city,
        district: geo.district,
        formatted_address: geo.address,
    })
}

fn coordinates_only(latitude: f64, longitude: f64) -> GpsResult {
    GpsResult {
        latitude,
        longitude,
        province: None,
        city: None,
        district: None,
        formatted_address: None,
    }
}

fn extract_gps(image_bytes: &[u8]) -> Result<(f64, f64), AppError> {
    let mut cursor = std::io::Cursor::new(image_bytes);
    let exif = exif::Reader::new()
        .read_from_container(&mut cursor)
        .map_err(|e| AppError::Internal(format!("EXIF read error: {e}")))?;

    let lat = exif
        .get_field(exif::Tag::GPSLatitude, exif::In::PRIMARY)
        .and_then(|f| gps_to_decimal(&f.value));
    let lat_ref = exif
        .get_field(exif::Tag::GPSLatitudeRef, exif::In::PRIMARY)
        .and_then(|f| f.display_value().to_string().into());

    let lon = exif
        .get_field(exif::Tag::GPSLongitude, exif::In::PRIMARY)
        .and_then(|f| gps_to_decimal(&f.value));
    let lon_ref = exif
        .get_field(exif::Tag::GPSLongitudeRef, exif::In::PRIMARY)
        .and_then(|f| f.display_value().to_string().into());

    match (lat, lon, lat_ref, lon_ref) {
        (Some(lat), Some(lon), lat_ref, lon_ref) => {
            let lat = if lat_ref.as_deref() == Some("S") { -lat } else { lat };
            let lon = if lon_ref.as_deref() == Some("W") { -lon } else { lon };
            Ok((lat, lon))
        }
        _ => Err(AppError::BadRequest("No GPS data found in image".into())),
    }
}

fn gps_to_decimal(value: &exif::Value) -> Option<f64> {
    match value {
        exif::Value::Rational(rationals) if rationals.len() >= 3 => {
            let d = rationals[0].to_f64();
            let m = rationals[1].to_f64();
            let s = rationals[2].to_f64();
            Some(d + m / 60.0 + s / 3600.0)
        }
        _ => None,
    }
}
