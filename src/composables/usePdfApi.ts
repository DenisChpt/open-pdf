import { invoke } from "@tauri-apps/api/core";
import type {
  CompressRequest,
  CompressResponse,
  MergeRequest,
  MergeResponse,
  PdfFileInfo,
  ReorderRequest,
  SplitBySizeRequest,
  SignRequest,
  SplitRequest,
  SplitResponse,
} from "../types/pdf";

export function usePdfApi() {
  async function getPdfInfo(path: string): Promise<PdfFileInfo> {
    return invoke<PdfFileInfo>("get_pdf_info", { path });
  }

  async function getPdfData(path: string): Promise<string> {
    return invoke<string>("get_pdf_data", { path });
  }

  async function mergePdfs(request: MergeRequest): Promise<MergeResponse> {
    return invoke<MergeResponse>("merge_pdfs", { request });
  }

  async function splitPdf(request: SplitRequest): Promise<SplitResponse> {
    return invoke<SplitResponse>("split_pdf", { request });
  }

  async function splitBySize(
    request: SplitBySizeRequest,
  ): Promise<SplitResponse> {
    return invoke<SplitResponse>("split_by_size", { request });
  }

  async function compressPdf(
    request: CompressRequest,
  ): Promise<CompressResponse> {
    return invoke<CompressResponse>("compress_pdf", { request });
  }

  async function reorderPages(request: ReorderRequest): Promise<void> {
    return invoke<void>("reorder_pages", { request });
  }

  async function saveFileBase64(path: string, dataBase64: string): Promise<void> {
    return invoke<void>("save_file_base64", {
      request: { path, data_base64: dataBase64 },
    });
  }

  async function signPdf(request: SignRequest): Promise<void> {
    return invoke<void>("sign_pdf", { request });
  }

  return {
    getPdfInfo,
    getPdfData,
    mergePdfs,
    splitPdf,
    splitBySize,
    compressPdf,
    reorderPages,
    signPdf,
    saveFileBase64,
  };
}
