use crate::error::AppError;

const NEEDS_FFMPEG_DECODE: &[&str] = &[
    ".heic", ".heif", ".avif", ".raw", ".cr2", ".cr3", ".nef", ".arw", ".dng", ".orf", ".rw2", ".pef", ".srw", ".raf",
];

pub async fn load_image_bytes(path: &str) -> Result<Vec<u8>, AppError> {
    let file_path = std::path::Path::new(path);
    if !file_path.exists() {
        return Err(AppError::NotFound(format!("file not found: {path}")));
    }

    let ext = file_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    let bytes = tokio::fs::read(file_path)
        .await
        .map_err(|e| AppError::Internal(format!("read file: {e}")))?;

    if NEEDS_FFMPEG_DECODE.contains(&format!(".{ext}").as_str()) {
        let filename = file_path.file_name().and_then(|f| f.to_str()).unwrap_or("image");
        convert_to_jpeg(bytes, filename).await
    } else {
        Ok(bytes)
    }
}

async fn convert_to_jpeg(bytes: Vec<u8>, filename: &str) -> Result<Vec<u8>, AppError> {
    let fname = filename.to_string();
    let result = tokio::task::spawn_blocking(move || {
        use tokimo_package_ffmpeg::image::{ImageDecodeOptions, ImageFormat, decode_image_from_bytes};

        let opts = ImageDecodeOptions {
            width: None,
            format: ImageFormat::Jpeg,
            quality: 2,
        };
        decode_image_from_bytes(&bytes, &fname, &opts)
    })
    .await
    .map_err(|e| AppError::Internal(format!("task join error: {e}")))?
    .map_err(|e| AppError::Internal(format!("FFI decode failed for {filename}: {e}")))?;

    Ok(result)
}
