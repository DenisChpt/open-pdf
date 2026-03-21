<script setup lang="ts">
import { ref, computed, watch } from "vue";
import DropZone from "../components/DropZone.vue";
import ActionButton from "../components/ActionButton.vue";
import StatusMessage from "../components/StatusMessage.vue";
import { usePdfApi } from "../composables/usePdfApi";
import { usePdfRenderer } from "../composables/usePdfRenderer";
import { useFileDialog } from "../composables/useFileDialog";
import type { SelectedFile } from "../types/pdf";

const { getPdfInfo, getPdfData, splitPdf, splitBySize, reorderPages } = usePdfApi();
const { renderThumbnails, isRendering } = usePdfRenderer();
const { pickPdfFiles, pickSavePath, pickDirectory } = useFileDialog();

// --- State ---

type SplitMode = "range" | "pages" | "size";
type RangeSubMode = "custom" | "fixed";
type PagesSubMode = "all" | "select";

interface RangeItem {
  id: number;
  start: number;
  end: number;
}

const file = ref<SelectedFile | null>(null);
const thumbnails = ref<string[]>([]);
const isLoading = ref(false);
const statusMessage = ref<{ type: "success" | "error"; message: string } | null>(null);

// Mode
const mode = ref<SplitMode>("range");
const rangeSubMode = ref<RangeSubMode>("custom");
const pagesSubMode = ref<PagesSubMode>("all");

// Range custom
const customRanges = ref<RangeItem[]>([]);
const mergeRanges = ref(false);
let rangeIdCounter = 0;

// Range fixed
const fixedInterval = ref(2);

// Pages
const selectedPages = ref<Set<number>>(new Set());
const mergeSelectedPages = ref(false);

// Size
const maxSizeValue = ref(500);
const sizeUnit = ref<"kb" | "mb">("kb");
const allowCompression = ref(false);

// --- Computed ---

const maxBytes = computed(() =>
  sizeUnit.value === "mb" ? maxSizeValue.value * 1024 * 1024 : maxSizeValue.value * 1024,
);

const totalPages = computed(() => file.value?.pageCount ?? 0);

const fixedChunkCount = computed(() => {
  if (totalPages.value === 0 || fixedInterval.value <= 0) return 0;
  return Math.ceil(totalPages.value / fixedInterval.value);
});

const GROUP_COLORS = [
  "#e74c3c", "#3498db", "#27ae60", "#9b59b6",
  "#f39c12", "#1abc9c", "#e67e22", "#0984e3",
];

const pageHighlights = computed(() => {
  const map = new Map<number, string>();
  if (!file.value) return map;

  if (mode.value === "range") {
    if (rangeSubMode.value === "custom") {
      customRanges.value.forEach((range, idx) => {
        const color = GROUP_COLORS[idx % GROUP_COLORS.length];
        for (let p = range.start; p <= Math.min(range.end, totalPages.value); p++) {
          if (p >= 1) map.set(p, color);
        }
      });
    } else {
      for (let i = 0; i < fixedChunkCount.value; i++) {
        const color = GROUP_COLORS[i % GROUP_COLORS.length];
        const start = i * fixedInterval.value + 1;
        const end = Math.min((i + 1) * fixedInterval.value, totalPages.value);
        for (let p = start; p <= end; p++) map.set(p, color);
      }
    }
  } else if (mode.value === "pages") {
    const color = GROUP_COLORS[1];
    if (pagesSubMode.value === "all") {
      for (let p = 1; p <= totalPages.value; p++) map.set(p, color);
    } else {
      selectedPages.value.forEach((p) => map.set(p, color));
    }
  }

  return map;
});

const canSplit = computed(() => {
  if (!file.value || isLoading.value) return false;
  if (mode.value === "range") {
    if (rangeSubMode.value === "custom") {
      return (
        customRanges.value.length > 0 &&
        customRanges.value.every(
          (r) => r.start >= 1 && r.end >= r.start && r.end <= totalPages.value,
        )
      );
    }
    return fixedInterval.value >= 1 && fixedInterval.value <= totalPages.value;
  }
  if (mode.value === "pages") {
    if (pagesSubMode.value === "all") return totalPages.value > 0;
    return selectedPages.value.size > 0;
  }
  if (mode.value === "size") {
    return maxSizeValue.value > 0;
  }
  return false;
});

// --- Actions ---

async function selectFile() {
  statusMessage.value = null;
  try {
    const paths = await pickPdfFiles();
    if (paths.length === 0) return;

    const info = await getPdfInfo(paths[0]);
    file.value = {
      path: info.path,
      name: info.file_name,
      pageCount: info.page_count,
      sizeBytes: info.file_size_bytes,
    };

    customRanges.value = [{ id: ++rangeIdCounter, start: 1, end: info.page_count }];
    selectedPages.value = new Set();
    thumbnails.value = [];

    const base64Data = await getPdfData(paths[0]);
    const thumbs = await renderThumbnails(base64Data, info.page_count);
    thumbnails.value = thumbs;
  } catch (error) {
    statusMessage.value = { type: "error", message: `Erreur : ${error}` };
  }
}

function addRange() {
  if (!file.value) return;
  customRanges.value.push({ id: ++rangeIdCounter, start: 1, end: file.value.pageCount });
}

function removeRange(index: number) {
  customRanges.value.splice(index, 1);
}

function togglePage(page: number) {
  if (pagesSubMode.value !== "select") return;
  const set = new Set(selectedPages.value);
  if (set.has(page)) set.delete(page);
  else set.add(page);
  selectedPages.value = set;
}

function selectAllPages() {
  const set = new Set<number>();
  for (let i = 1; i <= totalPages.value; i++) set.add(i);
  selectedPages.value = set;
}

function deselectAllPages() {
  selectedPages.value = new Set();
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} o`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} Ko`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} Mo`;
}

// Reset selection when switching modes
watch(pagesSubMode, (val) => {
  if (val === "all") selectedPages.value = new Set();
});

async function executeSplit() {
  if (!canSplit.value || !file.value) return;
  isLoading.value = true;
  statusMessage.value = null;

  try {
    // Modes that produce a single merged file
    const isMergeMode =
      (mode.value === "range" && rangeSubMode.value === "custom" && mergeRanges.value) ||
      (mode.value === "pages" && pagesSubMode.value === "select" && mergeSelectedPages.value);

    if (isMergeMode) {
      const outputPath = await pickSavePath(`${file.value.name}_extracted.pdf`);
      if (!outputPath) { isLoading.value = false; return; }

      let pageList: number[] = [];
      if (mode.value === "range") {
        const set = new Set<number>();
        for (const r of customRanges.value) {
          for (let p = r.start; p <= Math.min(r.end, totalPages.value); p++) set.add(p);
        }
        pageList = [...set].sort((a, b) => a - b);
      } else {
        pageList = [...selectedPages.value].sort((a, b) => a - b);
      }

      await reorderPages({
        input_path: file.value.path,
        new_order: pageList,
        output_path: outputPath,
      });

      statusMessage.value = { type: "success", message: `PDF cree avec ${pageList.length} pages` };
    } else if (mode.value === "size") {
      const outputDir = await pickDirectory();
      if (!outputDir) { isLoading.value = false; return; }

      const result = await splitBySize({
        input_path: file.value.path,
        max_bytes: maxBytes.value,
        compress: allowCompression.value,
        output_dir: outputDir,
      });

      statusMessage.value = {
        type: "success",
        message: `${result.output_paths.length} fichier${result.output_paths.length > 1 ? "s" : ""} cree${result.output_paths.length > 1 ? "s" : ""}`,
      };
    } else {
      // Multi-file split modes
      const outputDir = await pickDirectory();
      if (!outputDir) { isLoading.value = false; return; }

      let ranges: [number, number][] = [];

      if (mode.value === "range" && rangeSubMode.value === "custom") {
        ranges = customRanges.value.map((r) => [r.start, r.end]);
      } else if (mode.value === "range" && rangeSubMode.value === "fixed") {
        for (let i = 0; i < fixedChunkCount.value; i++) {
          const s = i * fixedInterval.value + 1;
          const e = Math.min((i + 1) * fixedInterval.value, totalPages.value);
          ranges.push([s, e]);
        }
      } else if (mode.value === "pages" && pagesSubMode.value === "all") {
        for (let p = 1; p <= totalPages.value; p++) ranges.push([p, p]);
      } else if (mode.value === "pages" && pagesSubMode.value === "select") {
        const sorted = [...selectedPages.value].sort((a, b) => a - b);
        for (const p of sorted) ranges.push([p, p]);
      }

      const result = await splitPdf({
        input_path: file.value.path,
        ranges,
        output_dir: outputDir,
      });

      statusMessage.value = {
        type: "success",
        message: `${result.output_paths.length} fichier${result.output_paths.length > 1 ? "s" : ""} cree${result.output_paths.length > 1 ? "s" : ""}`,
      };
    }
  } catch (error) {
    statusMessage.value = { type: "error", message: `Erreur : ${error}` };
  } finally {
    isLoading.value = false;
  }
}

function reset() {
  file.value = null;
  thumbnails.value = [];
  statusMessage.value = null;
  customRanges.value = [];
  selectedPages.value = new Set();
}
</script>

<template>
  <div class="split-view">
    <div class="view-header">
      <h1 class="view-title">Diviser PDF</h1>
      <p class="view-subtitle">Separarez votre PDF par intervalles, pages ou taille de fichier.</p>
    </div>

    <DropZone v-if="!file" label="Cliquez pour choisir un fichier PDF" @select="selectFile" />

    <template v-else>
      <!-- File info bar -->
      <div class="toolbar">
        <div class="toolbar-info">
          <span class="toolbar-filename">{{ file.name }}.pdf</span>
          <span class="toolbar-badge">{{ file.pageCount }} page{{ file.pageCount > 1 ? "s" : "" }}</span>
          <span class="toolbar-badge">{{ formatSize(file.sizeBytes) }}</span>
        </div>
        <button class="toolbar-btn" @click="reset">Changer</button>
      </div>

      <!-- Mode tabs -->
      <div class="mode-tabs">
        <button
          v-for="m in (['range', 'pages', 'size'] as const)"
          :key="m"
          class="mode-tab"
          :class="{ 'mode-tab--active': mode === m }"
          @click="mode = m"
        >
          {{ m === 'range' ? 'Intervalle' : m === 'pages' ? 'Pages' : 'Dimensions' }}
        </button>
      </div>

      <!-- Range Mode -->
      <div v-if="mode === 'range'" class="mode-panel">
        <div class="sub-tabs">
          <button
            class="sub-tab"
            :class="{ 'sub-tab--active': rangeSubMode === 'custom' }"
            @click="rangeSubMode = 'custom'"
          >Personnaliser</button>
          <button
            class="sub-tab"
            :class="{ 'sub-tab--active': rangeSubMode === 'fixed' }"
            @click="rangeSubMode = 'fixed'"
          >Fixe</button>
        </div>

        <!-- Custom ranges -->
        <template v-if="rangeSubMode === 'custom'">
          <div v-for="(range, index) in customRanges" :key="range.id" class="range-row">
            <span class="range-color" :style="{ background: GROUP_COLORS[index % GROUP_COLORS.length] }" />
            <label class="range-field">
              <span>De</span>
              <input v-model.number="range.start" type="number" :min="1" :max="totalPages" />
            </label>
            <label class="range-field">
              <span>A</span>
              <input v-model.number="range.end" type="number" :min="1" :max="totalPages" />
            </label>
            <span class="range-count">{{ Math.max(0, range.end - range.start + 1) }}p</span>
            <button v-if="customRanges.length > 1" class="range-remove" @click="removeRange(index)">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                <line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" />
              </svg>
            </button>
          </div>
          <div class="range-actions">
            <button class="add-btn" @click="addRange">+ Ajouter un intervalle</button>
            <label class="checkbox-label">
              <input v-model="mergeRanges" type="checkbox" />
              <span>Fusionner tous les intervalles dans un fichier PDF</span>
            </label>
          </div>
        </template>

        <!-- Fixed interval -->
        <template v-else>
          <div class="fixed-config">
            <label class="range-field">
              <span>Diviser toutes les</span>
              <input v-model.number="fixedInterval" type="number" :min="1" :max="totalPages" />
              <span>pages</span>
            </label>
          </div>
          <div v-if="fixedChunkCount > 0" class="info-message">
            Ce PDF sera divise en fichiers de {{ fixedInterval }} page{{ fixedInterval > 1 ? "s" : "" }}.
            <strong>{{ fixedChunkCount }} PDF</strong> seront crees.
          </div>
        </template>
      </div>

      <!-- Pages Mode -->
      <div v-if="mode === 'pages'" class="mode-panel">
        <div class="sub-tabs">
          <button
            class="sub-tab"
            :class="{ 'sub-tab--active': pagesSubMode === 'all' }"
            @click="pagesSubMode = 'all'"
          >Extraire toutes les pages</button>
          <button
            class="sub-tab"
            :class="{ 'sub-tab--active': pagesSubMode === 'select' }"
            @click="pagesSubMode = 'select'"
          >Selectionner les pages</button>
        </div>

        <template v-if="pagesSubMode === 'all'">
          <div class="info-message">
            Chaque page sera extraite dans un fichier PDF individuel.
            <strong>{{ totalPages }} PDF</strong> seront crees.
          </div>
        </template>
        <template v-else>
          <div class="select-actions">
            <button class="link-btn" @click="selectAllPages">Tout selectionner</button>
            <button class="link-btn" @click="deselectAllPages">Tout deselectionner</button>
            <span class="select-count">{{ selectedPages.size }} page{{ selectedPages.size > 1 ? "s" : "" }} selectionnee{{ selectedPages.size > 1 ? "s" : "" }}</span>
          </div>
          <label class="checkbox-label">
            <input v-model="mergeSelectedPages" type="checkbox" />
            <span>Fusionner les pages extraites dans un fichier PDF</span>
          </label>
        </template>
      </div>

      <!-- Size Mode -->
      <div v-if="mode === 'size'" class="mode-panel">
        <div class="size-config">
          <label class="range-field">
            <span>Taille maximale</span>
            <input v-model.number="maxSizeValue" type="number" :min="1" />
            <select v-model="sizeUnit" class="unit-select">
              <option value="kb">Ko</option>
              <option value="mb">Mo</option>
            </select>
          </label>
        </div>
        <label class="checkbox-label">
          <input v-model="allowCompression" type="checkbox" />
          <span>Autoriser la compression</span>
        </label>
        <div class="info-message">
          Ce PDF sera divise en fichiers de moins de
          <strong>{{ maxSizeValue }} {{ sizeUnit === 'kb' ? 'Ko' : 'Mo' }}</strong> chacun.
        </div>
      </div>

      <!-- Page thumbnails preview -->
      <div v-if="isRendering" class="loading-state">
        <div class="loading-spinner" />
        <p>Chargement des aperçus...</p>
      </div>

      <div v-else-if="thumbnails.length > 0" class="pages-grid">
        <div
          v-for="page in totalPages"
          :key="page"
          class="page-card"
          :class="{
            'page-card--selected': pageHighlights.has(page),
            'page-card--clickable': mode === 'pages' && pagesSubMode === 'select',
          }"
          @click="togglePage(page)"
        >
          <div
            class="page-thumbnail"
            :style="pageHighlights.has(page) ? { borderColor: pageHighlights.get(page) } : {}"
          >
            <img
              v-if="thumbnails[page - 1]"
              :src="thumbnails[page - 1]"
              :alt="`Page ${page}`"
              draggable="false"
            />
            <div v-else class="page-placeholder">{{ page }}</div>
            <!-- Checkmark overlay for selected pages in select mode -->
            <div
              v-if="mode === 'pages' && pagesSubMode === 'select' && selectedPages.has(page)"
              class="page-check"
              :style="{ background: pageHighlights.get(page) ?? '#3498db' }"
            >
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
                <polyline points="20 6 9 17 4 12" />
              </svg>
            </div>
            <!-- Color dot for range groups -->
            <span
              v-if="mode === 'range' && pageHighlights.has(page)"
              class="page-dot"
              :style="{ background: pageHighlights.get(page) }"
            />
          </div>
          <span class="page-label">{{ page }}</span>
        </div>
      </div>

      <!-- Actions -->
      <div class="actions">
        <ActionButton
          label="Diviser"
          :disabled="!canSplit"
          :loading="isLoading"
          @click="executeSplit"
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
.split-view {
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

/* Toolbar */
.toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-md) var(--space-lg);
  background: var(--color-bg-card);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-card);
}

.toolbar-info {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
}

.toolbar-filename {
  font-weight: 600;
  font-size: var(--font-size-sm);
}

.toolbar-badge {
  font-size: var(--font-size-xs);
  color: var(--color-text-secondary);
  padding: 2px var(--space-sm);
  background: var(--color-bg);
  border-radius: 100px;
}

.toolbar-btn {
  padding: var(--space-sm) var(--space-md);
  color: var(--color-text-secondary);
  font-size: var(--font-size-sm);
  font-weight: 500;
  border-radius: var(--radius-sm);
  transition: all var(--transition-fast);
}

.toolbar-btn:hover {
  background: var(--color-bg-hover);
  color: var(--color-text);
}

/* Mode tabs */
.mode-tabs {
  display: flex;
  background: var(--color-bg-card);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  padding: var(--space-xs);
  gap: var(--space-xs);
}

.mode-tab {
  flex: 1;
  padding: var(--space-sm) var(--space-md);
  font-size: var(--font-size-sm);
  font-weight: 600;
  border-radius: var(--radius-sm);
  color: var(--color-text-secondary);
  transition: all var(--transition-fast);
}

.mode-tab:hover {
  color: var(--color-text);
  background: var(--color-bg-hover);
}

.mode-tab--active {
  background: var(--color-primary);
  color: white;
}

.mode-tab--active:hover {
  background: var(--color-primary-hover);
  color: white;
}

/* Sub tabs */
.sub-tabs {
  display: flex;
  gap: var(--space-xs);
}

.sub-tab {
  padding: var(--space-sm) var(--space-md);
  font-size: var(--font-size-xs);
  font-weight: 600;
  border-radius: var(--radius-sm);
  color: var(--color-text-secondary);
  border: 1px solid var(--color-border);
  transition: all var(--transition-fast);
}

.sub-tab:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.sub-tab--active {
  background: var(--color-primary-light);
  border-color: var(--color-primary);
  color: var(--color-primary);
}

/* Mode panel */
.mode-panel {
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
  padding: var(--space-lg);
  background: var(--color-bg-card);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
}

/* Range rows */
.range-row {
  display: flex;
  align-items: center;
  gap: var(--space-md);
  padding: var(--space-sm) 0;
}

.range-color {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  flex-shrink: 0;
}

.range-field {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
}

.range-field input[type="number"] {
  width: 72px;
  padding: var(--space-sm);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  font-size: var(--font-size-sm);
  font-family: inherit;
  background: var(--color-bg);
  color: var(--color-text);
  text-align: center;
  outline: none;
  transition: border-color var(--transition-fast);
}

.range-field input:focus {
  border-color: var(--color-primary);
}

.range-count {
  font-size: var(--font-size-xs);
  color: var(--color-text-muted);
  min-width: 30px;
}

.range-remove {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  color: var(--color-text-muted);
  border-radius: var(--radius-sm);
  transition: all var(--transition-fast);
}

.range-remove:hover {
  background: var(--color-danger-light);
  color: var(--color-danger);
}

.range-actions {
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
  padding-top: var(--space-sm);
}

.add-btn {
  align-self: flex-start;
  padding: var(--space-sm) var(--space-md);
  font-size: var(--font-size-sm);
  font-weight: 600;
  color: var(--color-primary);
  background: var(--color-primary-light);
  border-radius: var(--radius-sm);
  transition: all var(--transition-fast);
}

.add-btn:hover {
  background: var(--color-primary);
  color: white;
}

/* Checkbox */
.checkbox-label {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
  cursor: pointer;
}

.checkbox-label input[type="checkbox"] {
  width: 16px;
  height: 16px;
  accent-color: var(--color-primary);
  cursor: pointer;
}

/* Fixed config */
.fixed-config {
  padding: var(--space-sm) 0;
}

/* Info message */
.info-message {
  padding: var(--space-md);
  background: var(--color-primary-light);
  color: var(--color-primary);
  border-radius: var(--radius-sm);
  font-size: var(--font-size-sm);
  line-height: 1.5;
}

.info-message strong {
  font-weight: 700;
}

/* Select actions */
.select-actions {
  display: flex;
  align-items: center;
  gap: var(--space-md);
}

.link-btn {
  font-size: var(--font-size-sm);
  font-weight: 600;
  color: var(--color-primary);
  transition: color var(--transition-fast);
}

.link-btn:hover {
  color: var(--color-primary-hover);
}

.select-count {
  margin-left: auto;
  font-size: var(--font-size-xs);
  color: var(--color-text-muted);
}

/* Size config */
.size-config {
  padding: var(--space-sm) 0;
}

.unit-select {
  padding: var(--space-sm) var(--space-md);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  font-size: var(--font-size-sm);
  font-family: inherit;
  background: var(--color-bg);
  color: var(--color-text);
  outline: none;
  cursor: pointer;
}

.unit-select:focus {
  border-color: var(--color-primary);
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
  user-select: none;
}

.page-card--clickable {
  cursor: pointer;
}

.page-thumbnail {
  position: relative;
  width: 100%;
  max-width: 200px;
  margin: 0 auto;
  aspect-ratio: 210 / 297;
  background: var(--color-bg-card);
  border: 2px solid var(--color-border);
  border-radius: var(--radius-md);
  overflow: hidden;
  box-shadow: var(--shadow-card);
  transition:
    border-color var(--transition-smooth),
    box-shadow var(--transition-smooth),
    transform var(--transition-smooth);
}

.page-card--clickable .page-thumbnail:hover {
  transform: translateY(-2px);
  box-shadow: var(--shadow-md);
}

.page-card--selected .page-thumbnail {
  box-shadow: var(--shadow-md);
}

.page-thumbnail img {
  width: 100%;
  height: 100%;
  object-fit: contain;
  background: white;
}

.page-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-bg);
  font-size: var(--font-size-lg);
  font-weight: 700;
  color: var(--color-text-muted);
}

.page-check {
  position: absolute;
  top: var(--space-xs);
  right: var(--space-xs);
  width: 22px;
  height: 22px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
}

.page-dot {
  position: absolute;
  bottom: var(--space-xs);
  left: var(--space-xs);
  width: 10px;
  height: 10px;
  border-radius: 50%;
}

.page-label {
  font-size: var(--font-size-xs);
  font-weight: 600;
  color: var(--color-text-secondary);
}

/* Actions */
.actions {
  display: flex;
  justify-content: flex-end;
}

@media (max-width: 600px) {
  .mode-tabs {
    flex-direction: column;
  }

  .sub-tabs {
    flex-direction: column;
  }

  .range-row {
    flex-wrap: wrap;
  }

  .toolbar-info {
    flex-wrap: wrap;
  }

  .select-actions {
    flex-wrap: wrap;
  }

  .size-config .range-field {
    flex-wrap: wrap;
  }
}
</style>
