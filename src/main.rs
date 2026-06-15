use std::path::PathBuf;
use std::sync::Arc;

use clap::Parser;
use tokimo_bus_cli::TokimoAuthArgs;
use tokimo_bus_client::{BusClient, ClientConfig};
use tokimo_perception::worker::client::AiWorkerClient;
use tracing::{error, info};

mod app_server;
mod assets;
mod cli;
mod config;
mod db;
mod error;
mod handlers;
mod services;
mod state;

use crate::db::repos::system_config_repo::SystemConfigSection;
use crate::state::AppState;

const MANIFEST: &str = include_str!("../tokimo-app.toml");

fn data_local_path() -> PathBuf {
    std::env::var("TOKIMO_DATA_LOCAL_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("./.data/local"))
}

#[derive(Parser, Debug)]
#[command(
    name = "tokimo-app-image-cortex",
    about = "Image Cortex — 图片分析中间件 (OCR / 人脸 / CLIP / GPS)",
    long_about = "Image Cortex CLI — analyze images with OCR, face detection, CLIP embedding, and GPS reverse geocoding.",
    term_width = 100
)]
struct Cli {
    #[command(flatten)]
    auth: TokimoAuthArgs,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(clap::Subcommand, Debug)]
enum Command {
    /// Analyze a single image
    Analyze {
        /// Image file path
        path: String,
        /// Analysis type: ocr, face, clip, gps, all
        #[arg(short, long, default_value = "all")]
        r#type: String,
    },
    /// Check AI worker health
    Health,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let Cli { auth: _, command } = Cli::parse();

    match command {
        None if std::env::var_os("TOKIMO_BUS_SOCKET").is_some() => {
            tracing_subscriber::fmt()
                .with_env_filter(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| "info,tokimo_bus_client=info,tokimo_app_image_cortex=debug".into()),
                )
                .init();
            if let Err(error) = run_server().await {
                error!(%error, "image-cortex: fatal");
                std::process::exit(1);
            }
        }
        None => {
            use clap::CommandFactory;
            let mut cmd = Cli::command();
            tokimo_bus_cli::print_help_unified(&mut cmd);
            std::process::exit(0);
        }
        Some(cmd) => match cmd {
            Command::Analyze { path, r#type } => {
                cli::run_analyze(path, r#type).await?;
            }
            Command::Health => {
                cli::run_health().await?;
            }
        },
    }

    Ok(())
}

async fn run_server() -> anyhow::Result<()> {
    let cfg = ClientConfig::from_env().map_err(|e| anyhow::anyhow!("ClientConfig: {e}"))?;
    info!(endpoint = ?cfg.endpoint, "image-cortex: connecting to broker");

    let db = db::init_pool().await?;
    info!("image-cortex: db connected (schema managed by host)");

    let _ai_settings: config::AiSettings = {
        use crate::db::repos::system_config_repo::SystemConfigRepo;
        SystemConfigRepo::get(&db)
            .await
            .unwrap_or_else(|_| config::AiSettings::default_value())
    };

    let ai_worker = AiWorkerClient::from_settings(
        &tokimo_perception::worker::client::AiWorkerSettings {
            mode: tokimo_perception::worker::client::AiWorkerMode::Auto,
            remote_url: None,
            keepalive_always: false,
            idle_timeout_secs: None,
            worker_binary: None,
            socket_path: None,
            models_dir: None,
        },
        &data_local_path(),
    );

    let ctx = Arc::new(AppState {
        db,
        ai_worker,
        http_client: reqwest::Client::new(),
    });

    let app_socket = app_server::spawn("image-cortex", Arc::clone(&ctx))
        .await
        .map_err(|e| anyhow::anyhow!("app_server spawn: {e}"))?;

    let client = BusClient::builder(cfg)
        .service("image-cortex", env!("CARGO_PKG_VERSION"))
        .data_plane(app_socket)
        .build()
        .await
        .map_err(|e| anyhow::anyhow!("bus build: {e}"))?;

    info!("image-cortex: registered with broker");

    let shutdown = {
        let client = Arc::clone(&client);
        tokio::spawn(async move { client.run_until_shutdown().await })
    };

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("image-cortex: SIGINT received");
            client.shutdown();
        }
        _ = shutdown => info!("image-cortex: broker sent Shutdown"),
    }

    Ok(())
}
