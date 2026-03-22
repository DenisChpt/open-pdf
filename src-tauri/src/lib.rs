mod application;
mod domain;
mod infrastructure;
mod interfaces;

use std::sync::Arc;

use application::use_cases::{
    CompressPdfUseCase, GetPdfDataUseCase, GetPdfInfoUseCase, MergePdfsUseCase,
    ReorderPagesUseCase, SaveFileUseCase, SignPdfUseCase, SplitBySizeUseCase, SplitPdfUseCase,
};
use domain::ports::{FileSystemPort, PdfProcessor};
use infrastructure::fs::LocalFileSystem;
use infrastructure::pdf::LopdfAdapter;
use interfaces::tauri_commands::{
    compress_pdf, delete_file, get_pdf_data, get_pdf_info, merge_pdfs, reorder_pages,
    save_file_base64, sign_pdf, split_by_size, split_pdf, AppState,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let pdf_processor: Arc<dyn PdfProcessor> = Arc::new(LopdfAdapter::new());
    let file_system: Arc<dyn FileSystemPort> = Arc::new(LocalFileSystem::new());

    let app_state = Arc::new(AppState {
        merge_pdfs: MergePdfsUseCase::new(Arc::clone(&pdf_processor), Arc::clone(&file_system)),
        split_pdf: SplitPdfUseCase::new(Arc::clone(&pdf_processor), Arc::clone(&file_system)),
        split_by_size: SplitBySizeUseCase::new(
            Arc::clone(&pdf_processor),
            Arc::clone(&file_system),
        ),
        get_pdf_info: GetPdfInfoUseCase::new(Arc::clone(&pdf_processor), Arc::clone(&file_system)),
        get_pdf_data: GetPdfDataUseCase::new(Arc::clone(&file_system)),
        compress_pdf: CompressPdfUseCase::new(Arc::clone(&pdf_processor), Arc::clone(&file_system)),
        reorder_pages: ReorderPagesUseCase::new(
            Arc::clone(&pdf_processor),
            Arc::clone(&file_system),
        ),
        sign_pdf: SignPdfUseCase::new(Arc::clone(&pdf_processor), Arc::clone(&file_system)),
        save_file: SaveFileUseCase::new(Arc::clone(&file_system)),
        file_system: Arc::clone(&file_system),
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            merge_pdfs,
            split_pdf,
            split_by_size,
            get_pdf_info,
            get_pdf_data,
            compress_pdf,
            reorder_pages,
            sign_pdf,
            save_file_base64,
            delete_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
