use std::sync::{Arc, OnceLock};

use sea_orm::DatabaseConnection;
use tokimo_bus_client::BusClient;
use tokimo_perception::worker::client::AiWorkerClient;

pub struct AppState {
    pub db: DatabaseConnection,
    pub ai_worker: Arc<AiWorkerClient>,
    pub http_client: reqwest::Client,
    pub bus_client: Arc<OnceLock<Arc<BusClient>>>,
}

impl AppState {
    pub fn models_ready(&self) -> bool {
        true
    }

    pub fn ocr_ready(&self) -> bool {
        true
    }

    pub fn face_ready(&self) -> bool {
        true
    }

    pub fn clip_ready(&self) -> bool {
        true
    }
}
