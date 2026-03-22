export interface PdfFileInfo {
  path: string;
  file_name: string;
  page_count: number;
  file_size_bytes: number;
}

export interface MergeRequest {
  input_paths: string[];
  output_path: string;
}

export interface MergeResponse {
  output_path: string;
  file_size_bytes: number;
}

export interface SplitRequest {
  input_path: string;
  ranges: [number, number][];
  output_dir: string;
}

export interface SplitResponse {
  output_paths: string[];
}

export interface CompressRequest {
  input_path: string;
  output_path: string;
}

export interface CompressResponse {
  original_size: number;
  compressed_size: number;
}

export interface ReorderRequest {
  input_path: string;
  new_order: number[];
  rotations?: Record<number, number>;
  output_path: string;
}

export interface SignElementDto {
  x_ratio: number;
  y_ratio: number;
  width_ratio: number;
  height_ratio: number;
  content:
    | { type: "text"; text: string; font_size: number }
    | { type: "image"; data_base64: string };
}

export interface SignRequest {
  input_path: string;
  output_path: string;
  page_number: number;
  elements: SignElementDto[];
}

export type SignatureItemType =
  | "name"
  | "initials"
  | "drawing"
  | "photo"
  | "stamp"
  | "lieu"
  | "date";

export interface SignatureItem {
  id: string;
  type: SignatureItemType;
  xRatio: number;
  yRatio: number;
  widthRatio: number;
  heightRatio: number;
  content: string;
  fontSize: number;
  grouped: boolean;
}

export interface SplitBySizeRequest {
  input_path: string;
  max_bytes: number;
  compress: boolean;
  output_dir: string;
}

export interface SelectedFile {
  path: string;
  name: string;
  pageCount: number;
  sizeBytes: number;
}

export interface PageItem {
  id: string;
  pageNumber: number;
  thumbnailUrl: string | null;
  rotation: number;
}
