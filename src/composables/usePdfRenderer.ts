import { ref } from "vue";
import * as pdfjsLib from "pdfjs-dist";

pdfjsLib.GlobalWorkerOptions.workerSrc = new URL(
  "pdfjs-dist/build/pdf.worker.min.mjs",
  import.meta.url,
).toString();

const DEFAULT_SCALE = 0.5;

export function usePdfRenderer() {
  const isRendering = ref(false);

  async function renderThumbnails(
    base64Data: string,
    pageCount: number,
    scale: number = DEFAULT_SCALE,
  ): Promise<string[]> {
    isRendering.value = true;

    try {
      const binaryData = atob(base64Data);
      const bytes = new Uint8Array(binaryData.length);
      for (let i = 0; i < binaryData.length; i++) {
        bytes[i] = binaryData.charCodeAt(i);
      }

      const pdf = await pdfjsLib.getDocument({ data: bytes }).promise;
      const thumbnails: string[] = [];

      for (let i = 1; i <= pageCount; i++) {
        const page = await pdf.getPage(i);
        const viewport = page.getViewport({ scale });

        const canvas = document.createElement("canvas");
        canvas.width = viewport.width;
        canvas.height = viewport.height;

        const context = canvas.getContext("2d");
        if (!context) {
          thumbnails.push("");
          continue;
        }

        await page.render({ canvas, viewport }).promise;
        thumbnails.push(canvas.toDataURL("image/png"));
      }

      return thumbnails;
    } finally {
      isRendering.value = false;
    }
  }

  return { renderThumbnails, isRendering };
}
