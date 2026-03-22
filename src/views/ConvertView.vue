<script setup lang="ts">
import { ref, computed } from "vue";
import * as pdfjsLib from "pdfjs-dist";
import DropZone from "../components/DropZone.vue";
import ActionButton from "../components/ActionButton.vue";
import StatusMessage from "../components/StatusMessage.vue";
import { usePdfApi } from "../composables/usePdfApi";
import { usePdfRenderer } from "../composables/usePdfRenderer";
import { useFileDialog } from "../composables/useFileDialog";
import type { SelectedFile } from "../types/pdf";

pdfjsLib.GlobalWorkerOptions.workerSrc = new URL(
  "pdfjs-dist/build/pdf.worker.min.mjs",
  import.meta.url,
).toString();

interface ImageFormat {
  id: string;
  label: string;
  ext: string;
  mime: string;
  hasQuality: boolean;
}

const FORMATS: ImageFormat[] = [
  { id: "png", label: "PNG", ext: "png", mime: "image/png", hasQuality: false },
  { id: "jpeg", label: "JPEG", ext: "jpg", mime: "image/jpeg", hasQuality: true },
  { id: "webp", label: "WEBP", ext: "webp", mime: "image/webp", hasQuality: true },
];

const SCALES = [
  { label: "72 DPI (1x)", value: 1 },
  { label: "144 DPI (2x)", value: 2 },
  { label: "216 DPI (3x)", value: 3 },
  { label: "300 DPI (4x)", value: 4 },
];

const { getPdfInfo, getPdfData, saveFileBase64 } = usePdfApi();
const { renderThumbnails, isRendering } = usePdfRenderer();
const { pickPdfFiles, pickDirectory } = useFileDialog();

const file = ref<SelectedFile | null>(null);
const thumbnails = ref<string[]>([]);
const format = ref<ImageFormat>(FORMATS[0]);
const quality = ref(90);
const scale = ref(2);
const isConverting = ref(false);
const progress = ref(0);
const statusMessage = ref<{ type: "success" | "error"; message: string } | null>(null);

// Store raw pdf bytes for rendering at high res
let pdfBase64Cache: string | null = null;

const qualityNormalized = computed(() => quality.value / 100);

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} o`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} Ko`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} Mo`;
}

async function loadPaths(paths: string[]) {
  if (paths.length === 0) return;
  statusMessage.value = null;
  try {
    const info = await getPdfInfo(paths[0]);
    file.value = {
      path: info.path,
      name: info.file_name,
      pageCount: info.page_count,
      sizeBytes: info.file_size_bytes,
    };
    thumbnails.value = [];
    const base64 = await getPdfData(paths[0]);
    pdfBase64Cache = base64;
    thumbnails.value = await renderThumbnails(base64, info.page_count);
  } catch (error) {
    statusMessage.value = { type: "error", message: `${error}` };
  }
}

async function selectFile() {
  const paths = await pickPdfFiles();
  await loadPaths(paths);
}

async function convert() {
  if (!file.value || !pdfBase64Cache || isConverting.value) return;
  isConverting.value = true;
  progress.value = 0;
  statusMessage.value = null;

  try {
    const outputDir = await pickDirectory();
    if (!outputDir) {
      isConverting.value = false;
      return;
    }

    // Load full PDF for hi-res rendering
    const binaryStr = atob(pdfBase64Cache);
    const bytes = new Uint8Array(binaryStr.length);
    for (let i = 0; i < binaryStr.length; i++) bytes[i] = binaryStr.charCodeAt(i);
    const pdf = await pdfjsLib.getDocument({ data: bytes }).promise;

    const total = file.value.pageCount;

    for (let pageNum = 1; pageNum <= total; pageNum++) {
      const page = await pdf.getPage(pageNum);
      const viewport = page.getViewport({ scale: scale.value });

      const canvas = document.createElement("canvas");
      canvas.width = viewport.width;
      canvas.height = viewport.height;

      // For JPEG: fill white background (no alpha support)
      if (format.value.id === "jpeg") {
        const bg = canvas.getContext("2d")!;
        bg.fillStyle = "#ffffff";
        bg.fillRect(0, 0, canvas.width, canvas.height);
      }

      await page.render({ canvas, viewport }).promise;

      // Export as data URL
      const dataUrl = format.value.hasQuality
        ? canvas.toDataURL(format.value.mime, qualityNormalized.value)
        : canvas.toDataURL(format.value.mime);

      // Strip "data:image/...;base64," prefix
      const base64Data = dataUrl.split(",")[1];

      const sep = outputDir.includes("\\") ? "\\" : "/";
      const filePath = `${outputDir}${sep}${file.value.name}_page${pageNum}.${format.value.ext}`;

      await saveFileBase64(filePath, base64Data);

      progress.value = Math.round((pageNum / total) * 100);
    }

    statusMessage.value = {
      type: "success",
      message: `${total} image${total > 1 ? "s" : ""} exportee${total > 1 ? "s" : ""} en ${format.value.label}`,
    };
  } catch (error) {
    statusMessage.value = { type: "error", message: `${error}` };
  } finally {
    isConverting.value = false;
  }
}

function reset() {
  file.value = null;
  thumbnails.value = [];
  pdfBase64Cache = null;
  statusMessage.value = null;
  progress.value = 0;
}
</script>

<template>
  <div class="convert-view">
    <div class="view-header">
      <h1 class="view-title">PDF vers Image</h1>
      <p class="view-subtitle">
        Convertissez chaque page de votre PDF en image haute qualite.
      </p>
    </div>

    <DropZone v-if="!file" label="Cliquez pour choisir un fichier PDF" @select="selectFile" @drop="loadPaths" />

    <template v-else>
      <!-- File info -->
      <div class="file-bar">
        <div class="file-bar-info">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
            <polyline points="14 2 14 8 20 8" />
          </svg>
          <div>
            <p class="file-bar-name">{{ file.name }}.pdf</p>
            <p class="file-bar-meta">
              {{ file.pageCount }} page{{ file.pageCount > 1 ? "s" : "" }}
              &middot; {{ formatSize(file.sizeBytes) }}
            </p>
          </div>
        </div>
        <button class="change-btn" @click="reset">Changer</button>
      </div>

      <!-- Settings -->
      <div class="settings">
        <!-- Format -->
        <div class="setting-group">
          <h3 class="setting-label">Format</h3>
          <div class="format-pills">
            <button
              v-for="f in FORMATS"
              :key="f.id"
              class="format-pill"
              :class="{ 'format-pill--active': format.id === f.id }"
              @click="format = f"
            >
              {{ f.label }}
            </button>
          </div>
        </div>

        <!-- Quality -->
        <div v-if="format.hasQuality" class="setting-group">
          <h3 class="setting-label">
            Qualite
            <span class="setting-value">{{ quality }}%</span>
          </h3>
          <input
            v-model.number="quality"
            type="range"
            :min="10"
            :max="100"
            :step="5"
            class="quality-slider"
          />
          <div class="quality-labels">
            <span>Legere</span>
            <span>Maximale</span>
          </div>
        </div>

        <!-- Scale -->
        <div class="setting-group">
          <h3 class="setting-label">Resolution</h3>
          <div class="scale-options">
            <button
              v-for="s in SCALES"
              :key="s.value"
              class="scale-btn"
              :class="{ 'scale-btn--active': scale === s.value }"
              @click="scale = s.value"
            >
              {{ s.label }}
            </button>
          </div>
        </div>
      </div>

      <!-- Page previews -->
      <div v-if="isRendering" class="loading-state">
        <div class="loading-spinner" />
        <p>Chargement des aperçus...</p>
      </div>
      <div v-else-if="thumbnails.length > 0" class="pages-grid">
        <div v-for="(thumb, i) in thumbnails" :key="i" class="page-card">
          <div class="page-thumb">
            <img :src="thumb" :alt="`Page ${i + 1}`" draggable="false" />
          </div>
          <span class="page-num">{{ i + 1 }}</span>
        </div>
      </div>

      <!-- Progress -->
      <div v-if="isConverting" class="progress-section">
        <div class="progress-bar">
          <div class="progress-fill" :style="{ width: `${progress}%` }" />
        </div>
        <span class="progress-text">{{ progress }}%</span>
      </div>

      <!-- Action -->
      <div class="actions">
        <ActionButton
          label="Convertir"
          :loading="isConverting"
          @click="convert"
        />
      </div>
    </template>

    <StatusMessage
      v-if="statusMessage"
      :type="statusMessage.type"
      :message="statusMessage.message"
    />
  </div>
</template>

<style scoped>
.convert-view {
  display: flex;
  flex-direction: column;
  gap: var(--space-lg);
  padding: var(--space-2xl) var(--page-padding);
  max-width: 900px;
  margin: 0 auto;
  width: 100%;
}

.view-header {
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
}

.view-title {
  font-size: var(--font-size-xl);
  font-weight: 700;
  letter-spacing: -0.02em;
}

.view-subtitle {
  color: var(--color-text-secondary);
  font-size: var(--font-size-sm);
}

/* File bar */
.file-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-md) var(--space-lg);
  background: var(--color-bg-card);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-card);
}

.file-bar-info {
  display: flex;
  align-items: center;
  gap: var(--space-md);
  color: var(--color-primary);
}

.file-bar-info > div {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.file-bar-name {
  font-weight: 600;
  font-size: var(--font-size-sm);
  color: var(--color-text);
}

.file-bar-meta {
  font-size: var(--font-size-xs);
  color: var(--color-text-secondary);
}

.change-btn {
  padding: var(--space-sm) var(--space-md);
  color: var(--color-text-secondary);
  border-radius: var(--radius-sm);
  font-size: var(--font-size-xs);
  font-weight: 500;
  transition: all var(--transition-fast);
}

.change-btn:hover {
  background: var(--color-bg-hover);
  color: var(--color-text);
}

/* Settings */
.settings {
  display: flex;
  flex-direction: column;
  gap: var(--space-lg);
  padding: var(--space-lg);
  background: var(--color-bg-card);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
}

.setting-group {
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
}

.setting-label {
  font-size: var(--font-size-xs);
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--color-text-secondary);
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.setting-value {
  font-size: var(--font-size-sm);
  font-weight: 600;
  color: var(--color-primary);
  text-transform: none;
  letter-spacing: 0;
}

/* Format pills */
.format-pills {
  display: flex;
  gap: var(--space-sm);
  flex-wrap: wrap;
}

.format-pill {
  padding: var(--space-sm) var(--space-lg);
  font-size: var(--font-size-sm);
  font-weight: 600;
  border: 1px solid var(--color-border);
  border-radius: 100px;
  color: var(--color-text-secondary);
  transition: all var(--transition-fast);
}

.format-pill:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.format-pill--active {
  background: var(--color-primary);
  border-color: var(--color-primary);
  color: white;
}

.format-pill--active:hover {
  background: var(--color-primary-hover);
  color: white;
}

/* Quality slider */
.quality-slider {
  width: 100%;
  height: 6px;
  appearance: none;
  background: var(--color-border);
  border-radius: 3px;
  outline: none;
  cursor: pointer;
}

.quality-slider::-webkit-slider-thumb {
  appearance: none;
  width: 18px;
  height: 18px;
  background: var(--color-primary);
  border-radius: 50%;
  cursor: pointer;
  border: 2px solid white;
  box-shadow: var(--shadow-sm);
}

.quality-labels {
  display: flex;
  justify-content: space-between;
  font-size: var(--font-size-xs);
  color: var(--color-text-muted);
}

/* Scale options */
.scale-options {
  display: flex;
  gap: var(--space-sm);
  flex-wrap: wrap;
}

.scale-btn {
  padding: var(--space-sm) var(--space-md);
  font-size: var(--font-size-xs);
  font-weight: 600;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  color: var(--color-text-secondary);
  transition: all var(--transition-fast);
}

.scale-btn:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.scale-btn--active {
  background: var(--color-primary-light);
  border-color: var(--color-primary);
  color: var(--color-primary);
}

/* Pages grid */
.pages-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(var(--grid-min-thumb), 1fr));
  gap: var(--space-md);
}

.page-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-xs);
}

.page-thumb {
  width: 100%;
  max-width: 200px;
  margin: 0 auto;
  aspect-ratio: 210 / 297;
  border: 2px solid var(--color-border);
  border-radius: var(--radius-md);
  overflow: hidden;
  box-shadow: var(--shadow-card);
  background: white;
}

.page-thumb img {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.page-num {
  font-size: var(--font-size-xs);
  font-weight: 600;
  color: var(--color-text-secondary);
}

/* Loading */
.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-md);
  padding: var(--space-2xl);
  color: var(--color-text-secondary);
}

.loading-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--color-border);
  border-top-color: var(--color-primary);
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* Progress */
.progress-section {
  display: flex;
  align-items: center;
  gap: var(--space-md);
}

.progress-bar {
  flex: 1;
  height: 8px;
  background: var(--color-border);
  border-radius: 4px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: var(--color-primary);
  border-radius: 4px;
  transition: width 200ms ease;
}

.progress-text {
  font-size: var(--font-size-sm);
  font-weight: 700;
  color: var(--color-primary);
  min-width: 40px;
  text-align: right;
}

/* Actions */
.actions {
  display: flex;
  justify-content: flex-end;
}
</style>
