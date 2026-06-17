use anyhow::Context;

use crate::{
    db::{init_pool, repos::system_config_repo::SystemConfigSection},
    services,
};

pub async fn run_analyze(path: String, analysis_type: String) -> anyhow::Result<()> {
    let db = init_pool().await.context("connect database failed")?;

    let _ai_settings: crate::config::AiSettings = {
        use crate::db::repos::system_config_repo::SystemConfigRepo;
        SystemConfigRepo::get(&db)
            .await
            .unwrap_or_else(|_| crate::config::AiSettings::default_value())
    };

    let data_local = std::env::var("TOKIMO_DATA_LOCAL_PATH")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("./.data/local"));

    let ai_worker = tokimo_perception::worker::client::AiWorkerClient::from_settings(
        &tokimo_perception::worker::client::AiWorkerSettings::default(),
        &data_local,
    );

    let http_client = reqwest::Client::new();

    let req = crate::handlers::analyze::AnalyzeRequest {
        path: path.clone(),
        analysis_type: match analysis_type.as_str() {
            "ocr" => crate::handlers::analyze::AnalysisType::Ocr,
            "face" => crate::handlers::analyze::AnalysisType::Face,
            "clip" => crate::handlers::analyze::AnalysisType::Clip,
            "gps" => crate::handlers::analyze::AnalysisType::Gps,
            _ => crate::handlers::analyze::AnalysisType::All,
        },
    };

    let image_bytes = services::image_loader::load_image_bytes(&http_client, &req.path).await?;

    let response = match req.analysis_type {
        crate::handlers::analyze::AnalysisType::Ocr => {
            let ocr = services::ocr::analyze(&ai_worker, image_bytes).await?;
            serde_json::json!({ "path": req.path, "ocr": ocr })
        }
        crate::handlers::analyze::AnalysisType::Face => {
            let face = services::face::analyze(&ai_worker, image_bytes).await?;
            serde_json::json!({ "path": req.path, "face": face })
        }
        crate::handlers::analyze::AnalysisType::Clip => {
            let clip = services::clip::analyze(&ai_worker, image_bytes).await?;
            serde_json::json!({ "path": req.path, "clip": clip })
        }
        crate::handlers::analyze::AnalysisType::Gps => {
            let geo_settings: crate::config::GeoSettings = {
                use crate::db::repos::system_config_repo::SystemConfigRepo;
                SystemConfigRepo::get(&db)
                    .await
                    .unwrap_or_else(|_| crate::config::GeoSettings::default_value())
            };
            let gps = services::geo::analyze(&http_client, &image_bytes, &geo_settings).await?;
            serde_json::json!({ "path": req.path, "gps": gps })
        }
        crate::handlers::analyze::AnalysisType::All => {
            let geo_settings = crate::config::GeoSettings::default_value();
            let (ocr_r, face_r, clip_r, gps_r) = tokio::join!(
                services::ocr::analyze(&ai_worker, image_bytes.clone()),
                services::face::analyze(&ai_worker, image_bytes.clone()),
                services::clip::analyze(&ai_worker, image_bytes.clone()),
                services::geo::analyze(&http_client, &image_bytes, &geo_settings),
            );
            serde_json::json!({
                "path": req.path,
                "ocr": ocr_r.ok(),
                "face": face_r.ok(),
                "clip": clip_r.ok(),
                "gps": gps_r.ok(),
            })
        }
    };

    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}

pub async fn run_health() -> anyhow::Result<()> {
    println!("Image Cortex CLI — health check");
    println!("Use 'bun dev --apps=image-cortex' to run the full server.");
    Ok(())
}
