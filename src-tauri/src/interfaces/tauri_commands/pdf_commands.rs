use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::State;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;

use crate::application::use_cases::{
    CompressPdfUseCase, GetPdfDataUseCase, GetPdfInfoUseCase, MergePdfsUseCase,
    ReorderPagesUseCase, SaveFileUseCase, SignPdfUseCase, SplitBySizeUseCase, SplitPdfUseCase,
};
use crate::domain::entities::{OverlayContent, OverlayElement};
use crate::domain::errors::DomainError;

/// Shared application state holding the use cases, injected via Tauri's managed state.
pub struct AppState {
    pub merge_pdfs: MergePdfsUseCase,
    pub split_pdf: SplitPdfUseCase,
    pub split_by_size: SplitBySizeUseCase,
    pub get_pdf_info: GetPdfInfoUseCase,
    pub get_pdf_data: GetPdfDataUseCase,
    pub compress_pdf: CompressPdfUseCase,
    pub reorder_pages: ReorderPagesUseCase,
    pub sign_pdf: SignPdfUseCase,
    pub save_file: SaveFileUseCase,
}

// -- PDF Info --

#[derive(Serialize)]
pub struct PdfInfoResponse {
    pub path: PathBuf,
    pub file_name: String,
    pub page_count: u32,
    pub file_size_bytes: u64,
}

#[tauri::command]
pub async fn get_pdf_info(
    state: State<'_, Arc<AppState>>,
    path: PathBuf,
) -> Result<PdfInfoResponse, DomainError> {
    let state = Arc::clone(&state);
    let doc = tokio::task::spawn_blocking(move || state.get_pdf_info.execute(&path))
        .await
        .map_err(|e| DomainError::ProcessingError {
            reason: format!("Task join error: {}", e),
        })??;

    Ok(PdfInfoResponse {
        file_name: doc.file_name(),
        path: doc.path,
        page_count: doc.page_count,
        file_size_bytes: doc.file_size_bytes,
    })
}

// -- PDF Data (base64 for pdf.js) --

#[tauri::command]
pub async fn get_pdf_data(
    state: State<'_, Arc<AppState>>,
    path: PathBuf,
) -> Result<String, DomainError> {
    let state = Arc::clone(&state);
    tokio::task::spawn_blocking(move || state.get_pdf_data.execute(&path))
        .await
        .map_err(|e| DomainError::ProcessingError {
            reason: format!("Task join error: {}", e),
        })?
}

// -- Merge --

#[derive(Deserialize)]
pub struct MergeRequest {
    pub input_paths: Vec<PathBuf>,
    pub output_path: PathBuf,
}

#[derive(Serialize)]
pub struct MergeResponse {
    pub output_path: PathBuf,
    pub file_size_bytes: u64,
}

#[tauri::command]
pub async fn merge_pdfs(
    state: State<'_, Arc<AppState>>,
    request: MergeRequest,
) -> Result<MergeResponse, DomainError> {
    let state = Arc::clone(&state);
    let output_path = request.output_path.clone();

    let file_size_bytes = tokio::task::spawn_blocking(move || {
        state
            .merge_pdfs
            .execute(&request.input_paths, &request.output_path)
    })
    .await
    .map_err(|e| DomainError::ProcessingError {
        reason: format!("Task join error: {}", e),
    })??;

    Ok(MergeResponse {
        output_path,
        file_size_bytes,
    })
}

// -- Split --

#[derive(Deserialize)]
pub struct SplitRequest {
    pub input_path: PathBuf,
    pub ranges: Vec<(u32, u32)>,
    pub output_dir: PathBuf,
}

#[derive(Serialize)]
pub struct SplitResponse {
    pub output_paths: Vec<PathBuf>,
}

#[tauri::command]
pub async fn split_pdf(
    state: State<'_, Arc<AppState>>,
    request: SplitRequest,
) -> Result<SplitResponse, DomainError> {
    let state = Arc::clone(&state);

    let output_paths = tokio::task::spawn_blocking(move || {
        state
            .split_pdf
            .execute(&request.input_path, &request.ranges, &request.output_dir)
    })
    .await
    .map_err(|e| DomainError::ProcessingError {
        reason: format!("Task join error: {}", e),
    })??;

    Ok(SplitResponse { output_paths })
}

// -- Sign PDF --

#[derive(Deserialize)]
pub struct SignElementDto {
    pub x_ratio: f32,
    pub y_ratio: f32,
    pub width_ratio: f32,
    pub height_ratio: f32,
    pub content: SignContentDto,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum SignContentDto {
    #[serde(rename = "text")]
    Text { text: String, font_size: f32 },
    #[serde(rename = "image")]
    Image { data_base64: String },
}

#[derive(Deserialize)]
pub struct SignRequest {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub page_number: u32,
    pub elements: Vec<SignElementDto>,
}

#[tauri::command]
pub async fn sign_pdf(
    state: State<'_, Arc<AppState>>,
    request: SignRequest,
) -> Result<(), DomainError> {
    let state = Arc::clone(&state);

    tokio::task::spawn_blocking(move || {
        let elements: Vec<OverlayElement> = request
            .elements
            .into_iter()
            .map(|e| {
                let content = match e.content {
                    SignContentDto::Text { text, font_size } => {
                        OverlayContent::Text { text, font_size }
                    }
                    SignContentDto::Image { data_base64 } => {
                        let data = STANDARD.decode(&data_base64).map_err(|err| {
                            DomainError::ProcessingError {
                                reason: format!("Invalid base64 image data: {}", err),
                            }
                        })?;
                        OverlayContent::Image { data }
                    }
                };
                Ok(OverlayElement {
                    x_ratio: e.x_ratio,
                    y_ratio: e.y_ratio,
                    width_ratio: e.width_ratio,
                    height_ratio: e.height_ratio,
                    content,
                })
            })
            .collect::<Result<Vec<_>, DomainError>>()?;

        state.sign_pdf.execute(
            &request.input_path,
            &request.output_path,
            request.page_number,
            &elements,
        )
    })
    .await
    .map_err(|e| DomainError::ProcessingError {
        reason: format!("Task join error: {}", e),
    })?
}

// -- Compress --

#[derive(Deserialize)]
pub struct CompressRequest {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
}

#[derive(Serialize)]
pub struct CompressResponse {
    pub original_size: u64,
    pub compressed_size: u64,
}

#[tauri::command]
pub async fn compress_pdf(
    state: State<'_, Arc<AppState>>,
    request: CompressRequest,
) -> Result<CompressResponse, DomainError> {
    let state = Arc::clone(&state);

    let (original_size, compressed_size) = tokio::task::spawn_blocking(move || {
        state
            .compress_pdf
            .execute(&request.input_path, &request.output_path)
    })
    .await
    .map_err(|e| DomainError::ProcessingError {
        reason: format!("Task join error: {}", e),
    })??;

    Ok(CompressResponse {
        original_size,
        compressed_size,
    })
}

// -- Reorder Pages --

#[derive(Deserialize)]
pub struct ReorderRequest {
    pub input_path: PathBuf,
    pub new_order: Vec<u32>,
    #[serde(default)]
    pub rotations: std::collections::HashMap<u32, u32>,
    pub output_path: PathBuf,
}

#[tauri::command]
pub async fn reorder_pages(
    state: State<'_, Arc<AppState>>,
    request: ReorderRequest,
) -> Result<(), DomainError> {
    let state = Arc::clone(&state);

    tokio::task::spawn_blocking(move || {
        state.reorder_pages.execute(
            &request.input_path,
            &request.new_order,
            &request.rotations,
            &request.output_path,
        )
    })
    .await
    .map_err(|e| DomainError::ProcessingError {
        reason: format!("Task join error: {}", e),
    })?
}

// -- Split by Size --

#[derive(Deserialize)]
pub struct SplitBySizeRequest {
    pub input_path: PathBuf,
    pub max_bytes: u64,
    pub compress: bool,
    pub output_dir: PathBuf,
}

#[tauri::command]
pub async fn split_by_size(
    state: State<'_, Arc<AppState>>,
    request: SplitBySizeRequest,
) -> Result<SplitResponse, DomainError> {
    let state = Arc::clone(&state);

    let output_paths = tokio::task::spawn_blocking(move || {
        state.split_by_size.execute(
            &request.input_path,
            request.max_bytes,
            request.compress,
            &request.output_dir,
        )
    })
    .await
    .map_err(|e| DomainError::ProcessingError {
        reason: format!("Task join error: {}", e),
    })??;

    Ok(SplitResponse { output_paths })
}

// -- Save File (for PDF-to-image conversion) --

#[derive(Deserialize)]
pub struct SaveFileBase64Request {
    pub path: PathBuf,
    pub data_base64: String,
}

#[tauri::command]
pub async fn save_file_base64(
    state: State<'_, Arc<AppState>>,
    request: SaveFileBase64Request,
) -> Result<(), DomainError> {
    let state = Arc::clone(&state);

    tokio::task::spawn_blocking(move || {
        let data = STANDARD.decode(&request.data_base64).map_err(|e| {
            DomainError::ProcessingError {
                reason: format!("Invalid base64: {}", e),
            }
        })?;
        state.save_file.execute(&request.path, &data)
    })
    .await
    .map_err(|e| DomainError::ProcessingError {
        reason: format!("Task join error: {}", e),
    })?
}
