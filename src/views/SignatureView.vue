<script setup lang="ts">
import { ref, computed, watch } from "vue";
import DropZone from "../components/DropZone.vue";
import DrawingPad from "../components/DrawingPad.vue";
import ActionButton from "../components/ActionButton.vue";
import StatusMessage from "../components/StatusMessage.vue";
import { usePdfApi } from "../composables/usePdfApi";
import { usePdfRenderer } from "../composables/usePdfRenderer";
import { useFileDialog } from "../composables/useFileDialog";
import type { SelectedFile, SignatureItem, SignElementDto } from "../types/pdf";

const { getPdfInfo, getPdfData, signPdf } = usePdfApi();
const { renderThumbnails, isRendering } = usePdfRenderer();
const { pickPdfFiles, pickSavePath } = useFileDialog();

// --- File state ---
const file = ref<SelectedFile | null>(null);
const thumbnails = ref<string[]>([]);
const currentPage = ref(1);
const isLoading = ref(false);
const statusMessage = ref<{ type: "success" | "error"; message: string } | null>(null);

// --- Signature elements on preview ---
const items = ref<SignatureItem[]>([]);

// --- Palette state ---
const firstName = ref("");
const lastName = ref("");
const initialsOverride = ref("");
const dateFormat = ref("DD/MM/YYYY");
const drawingDataUrl = ref<string | null>(null);
const photoDataUrl = ref<string | null>(null);
const stampDataUrl = ref<string | null>(null);

// --- Drag state ---
const dragState = ref<{
  itemId: string;
  offsetX: number;
  offsetY: number;
} | null>(null);
const previewRef = ref<HTMLDivElement>();

const initials = computed(() => {
  if (initialsOverride.value) return initialsOverride.value;
  const parts: string[] = [];
  if (firstName.value) parts.push(firstName.value[0].toUpperCase());
  if (lastName.value) parts.push(lastName.value[0].toUpperCase());
  return parts.join("");
});

const formattedDate = computed(() => {
  const now = new Date();
  const d = String(now.getDate()).padStart(2, "0");
  const m = String(now.getMonth() + 1).padStart(2, "0");
  const y = now.getFullYear();
  const months = [
    "janvier", "fevrier", "mars", "avril", "mai", "juin",
    "juillet", "aout", "septembre", "octobre", "novembre", "decembre",
  ];
  switch (dateFormat.value) {
    case "MM/DD/YYYY": return `${m}/${d}/${y}`;
    case "YYYY-MM-DD": return `${y}-${m}-${d}`;
    case "DD MMMM YYYY": return `${now.getDate()} ${months[now.getMonth()]} ${y}`;
    default: return `${d}/${m}/${y}`;
  }
});

const allGrouped = computed(() =>
  items.value.length > 1 && items.value.every((i) => i.grouped),
);

let itemIdCounter = 0;

function makeId(): string {
  return `sig-${++itemIdCounter}`;
}

// --- File selection ---
async function selectFile() {
  statusMessage.value = null;
  try {
    const paths = await pickPdfFiles();
    if (paths.length === 0) return;
    const info = await getPdfInfo(paths[0]);
    file.value = {
      path: info.path, name: info.file_name,
      pageCount: info.page_count, sizeBytes: info.file_size_bytes,
    };
    currentPage.value = 1;
    items.value = [];
    thumbnails.value = [];

    const base64 = await getPdfData(paths[0]);
    thumbnails.value = await renderThumbnails(base64, info.page_count);
  } catch (error) {
    statusMessage.value = { type: "error", message: `${error}` };
  }
}

// --- Add elements ---
function addTextItem(
  type: SignatureItem["type"],
  text: string,
  fontSize: number,
  wRatio: number,
  hRatio: number,
) {
  if (!text) return;
  items.value.push({
    id: makeId(), type, content: text, fontSize,
    xRatio: 0.1, yRatio: 0.8,
    widthRatio: wRatio, heightRatio: hRatio, grouped: false,
  });
}

function addImageItem(type: SignatureItem["type"], dataUrl: string | null) {
  if (!dataUrl) return;
  items.value.push({
    id: makeId(), type, content: dataUrl, fontSize: 0,
    xRatio: 0.1, yRatio: 0.7,
    widthRatio: 0.25, heightRatio: 0.08, grouped: false,
  });
}

function addName() { addTextItem("name", `${firstName.value} ${lastName.value}`.trim(), 14, 0.25, 0.03); }
function addInitials() { addTextItem("initials", initials.value, 14, 0.06, 0.03); }
function addDate() { addTextItem("date", formattedDate.value, 11, 0.18, 0.025); }
function addDrawing() { addImageItem("drawing", drawingDataUrl.value); }
function addPhoto() { addImageItem("photo", photoDataUrl.value); }
function addStamp() { addImageItem("stamp", stampDataUrl.value); }

function removeItem(id: string) {
  items.value = items.value.filter((i) => i.id !== id);
}

// --- Photo/stamp upload ---
function uploadImage(target: "photo" | "stamp") {
  const input = document.createElement("input");
  input.type = "file";
  input.accept = "image/png,image/jpeg,image/webp";
  input.onchange = () => {
    const f = input.files?.[0];
    if (!f) return;
    const reader = new FileReader();
    reader.onload = () => {
      const result = reader.result as string;
      if (target === "photo") photoDataUrl.value = result;
      else stampDataUrl.value = result;
    };
    reader.readAsDataURL(f);
  };
  input.click();
}

// --- Drag on preview ---
function onItemPointerDown(e: PointerEvent, item: SignatureItem) {
  e.preventDefault();
  const preview = previewRef.value;
  if (!preview) return;
  const rect = preview.getBoundingClientRect();
  dragState.value = {
    itemId: item.id,
    offsetX: e.clientX - item.xRatio * rect.width,
    offsetY: e.clientY - item.yRatio * rect.height,
  };
  (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
}

function onPreviewPointerMove(e: PointerEvent) {
  if (!dragState.value) return;
  const preview = previewRef.value;
  if (!preview) return;
  const rect = preview.getBoundingClientRect();

  const movedItem = items.value.find((i) => i.id === dragState.value!.itemId);
  if (!movedItem) return;

  const newX = (e.clientX - dragState.value.offsetX) / rect.width;
  const newY = (e.clientY - dragState.value.offsetY) / rect.height;

  if (movedItem.grouped && allGrouped.value) {
    const dx = newX - movedItem.xRatio;
    const dy = newY - movedItem.yRatio;
    for (const item of items.value) {
      if (item.grouped) {
        item.xRatio = clamp(item.xRatio + dx, 0, 1 - item.widthRatio);
        item.yRatio = clamp(item.yRatio + dy, 0, 1 - item.heightRatio);
      }
    }
  } else {
    movedItem.xRatio = clamp(newX, 0, 1 - movedItem.widthRatio);
    movedItem.yRatio = clamp(newY, 0, 1 - movedItem.heightRatio);
  }
}

function onPreviewPointerUp() {
  dragState.value = null;
}

function clamp(v: number, min: number, max: number): number {
  return Math.min(Math.max(v, min), max);
}

// --- Grouping ---
function toggleGroup() {
  const shouldGroup = !allGrouped.value;
  items.value.forEach((i) => (i.grouped = shouldGroup));
}

// --- Apply signature ---
async function apply() {
  if (!file.value || items.value.length === 0) return;
  isLoading.value = true;
  statusMessage.value = null;

  try {
    const outputPath = await pickSavePath(`${file.value.name}_signed.pdf`);
    if (!outputPath) { isLoading.value = false; return; }

    const elements: SignElementDto[] = items.value.map((item) => {
      const isImage = ["drawing", "photo", "stamp"].includes(item.type);
      if (isImage) {
        // Strip data URL prefix to get raw base64
        const base64 = item.content.includes(",")
          ? item.content.split(",")[1]
          : item.content;
        return {
          x_ratio: item.xRatio,
          y_ratio: item.yRatio,
          width_ratio: item.widthRatio,
          height_ratio: item.heightRatio,
          content: { type: "image" as const, data_base64: base64 },
        };
      }
      return {
        x_ratio: item.xRatio,
        y_ratio: item.yRatio,
        width_ratio: item.widthRatio,
        height_ratio: item.heightRatio,
        content: { type: "text" as const, text: item.content, font_size: item.fontSize },
      };
    });

    await signPdf({
      input_path: file.value.path,
      output_path: outputPath,
      page_number: currentPage.value,
      elements,
    });

    statusMessage.value = { type: "success", message: "PDF signe avec succes" };
  } catch (error) {
    statusMessage.value = { type: "error", message: `${error}` };
  } finally {
    isLoading.value = false;
  }
}

function reset() {
  file.value = null;
  thumbnails.value = [];
  items.value = [];
  statusMessage.value = null;
}

// Unused var cleanup for the watch
watch(firstName, () => { initialsOverride.value = ""; });
watch(lastName, () => { initialsOverride.value = ""; });
</script>

<template>
  <div class="sign-view">
    <div class="view-header">
      <h1 class="view-title">Signer PDF</h1>
      <p class="view-subtitle">
        Ajoutez votre nom, initiales, signature, tampon ou date sur votre document.
      </p>
    </div>

    <DropZone v-if="!file" label="Cliquez pour choisir un fichier PDF" @select="selectFile" />

    <template v-else>
      <div class="sign-layout">
        <!-- Sidebar palette -->
        <aside class="palette">
          <!-- Name -->
          <section class="palette-section">
            <h3 class="palette-title">Nom et Prenom</h3>
            <div class="field-row">
              <input v-model="firstName" placeholder="Prenom" class="palette-input" />
              <input v-model="lastName" placeholder="Nom" class="palette-input" />
            </div>
            <button class="palette-add" :disabled="!firstName && !lastName" @click="addName">
              + Ajouter le nom
            </button>
          </section>

          <!-- Initials -->
          <section class="palette-section">
            <h3 class="palette-title">Initiales</h3>
            <input
              v-model="initialsOverride"
              :placeholder="initials || 'AB'"
              class="palette-input palette-input--short"
            />
            <button class="palette-add" :disabled="!initials" @click="addInitials">
              + Ajouter les initiales
            </button>
          </section>

          <!-- Drawing -->
          <section class="palette-section">
            <h3 class="palette-title">Signature manuscrite</h3>
            <DrawingPad @drawn="(url) => (drawingDataUrl = url)" />
            <button class="palette-add" :disabled="!drawingDataUrl" @click="addDrawing">
              + Ajouter la signature
            </button>
          </section>

          <!-- Photo -->
          <section class="palette-section">
            <h3 class="palette-title">Photo de signature</h3>
            <div class="upload-row">
              <button class="upload-btn" @click="uploadImage('photo')">Charger une image</button>
              <img v-if="photoDataUrl" :src="photoDataUrl" class="upload-preview" alt="Photo" />
            </div>
            <button class="palette-add" :disabled="!photoDataUrl" @click="addPhoto">
              + Ajouter la photo
            </button>
          </section>

          <!-- Stamp -->
          <section class="palette-section">
            <h3 class="palette-title">Tampon d'entreprise</h3>
            <div class="upload-row">
              <button class="upload-btn" @click="uploadImage('stamp')">Charger un tampon</button>
              <img v-if="stampDataUrl" :src="stampDataUrl" class="upload-preview" alt="Tampon" />
            </div>
            <button class="palette-add" :disabled="!stampDataUrl" @click="addStamp">
              + Ajouter le tampon
            </button>
          </section>

          <!-- Date -->
          <section class="palette-section">
            <h3 class="palette-title">Date</h3>
            <div class="field-row">
              <span class="date-preview">{{ formattedDate }}</span>
              <select v-model="dateFormat" class="palette-select">
                <option value="DD/MM/YYYY">DD/MM/YYYY</option>
                <option value="MM/DD/YYYY">MM/DD/YYYY</option>
                <option value="YYYY-MM-DD">YYYY-MM-DD</option>
                <option value="DD MMMM YYYY">DD MMMM YYYY</option>
              </select>
            </div>
            <button class="palette-add" @click="addDate">+ Ajouter la date</button>
          </section>

          <!-- Group button -->
          <button
            v-if="items.length > 1"
            class="group-btn"
            :class="{ 'group-btn--active': allGrouped }"
            @click="toggleGroup"
          >
            {{ allGrouped ? "Degrouper les elements" : "Grouper tous les elements" }}
          </button>
        </aside>

        <!-- Preview -->
        <div class="preview-area">
          <div class="preview-toolbar">
            <button :disabled="currentPage <= 1" class="page-btn" @click="currentPage--">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="15 18 9 12 15 6"/></svg>
            </button>
            <span class="page-indicator">
              {{ currentPage }} / {{ file.pageCount }}
            </span>
            <button :disabled="currentPage >= file.pageCount" class="page-btn" @click="currentPage++">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="9 18 15 12 9 6"/></svg>
            </button>
            <button class="toolbar-btn" @click="reset">Changer</button>
          </div>

          <div
            v-if="isRendering"
            class="loading-state"
          >
            <div class="loading-spinner" />
          </div>
          <div
            v-else
            ref="previewRef"
            class="preview-container"
            @pointermove="onPreviewPointerMove"
            @pointerup="onPreviewPointerUp"
          >
            <img
              v-if="thumbnails[currentPage - 1]"
              :src="thumbnails[currentPage - 1]"
              class="preview-page"
              draggable="false"
              alt="PDF page"
            />

            <!-- Overlaid signature items -->
            <div
              v-for="item in items"
              :key="item.id"
              class="sig-item"
              :class="{
                'sig-item--grouped': item.grouped && allGrouped,
                'sig-item--dragging': dragState?.itemId === item.id,
              }"
              :style="{
                left: `${item.xRatio * 100}%`,
                top: `${item.yRatio * 100}%`,
                width: `${item.widthRatio * 100}%`,
                height: `${item.heightRatio * 100}%`,
              }"
              @pointerdown="onItemPointerDown($event, item)"
            >
              <!-- Text content -->
              <span
                v-if="['name', 'initials', 'date'].includes(item.type)"
                class="sig-text"
                :style="{ fontSize: `${item.fontSize * 0.8}px` }"
              >{{ item.content }}</span>

              <!-- Image content -->
              <img
                v-else
                :src="item.content"
                class="sig-img"
                draggable="false"
                alt=""
              />

              <button class="sig-delete" @click.stop="removeItem(item.id)">
                <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
              </button>
            </div>
          </div>
        </div>
      </div>

      <div class="actions">
        <ActionButton
          label="Appliquer la signature"
          :disabled="items.length === 0"
          :loading="isLoading"
          @click="apply"
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
.sign-view {
  display: flex;
  flex-direction: column;
  gap: var(--space-lg);
  padding: var(--space-2xl) var(--page-padding);
  max-width: 1100px;
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

/* Layout */
.sign-layout {
  display: grid;
  grid-template-columns: 280px 1fr;
  gap: var(--space-lg);
  align-items: start;
}

/* Palette */
.palette {
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
  max-height: calc(100vh - 180px);
  overflow-y: auto;
}

.palette-section {
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
  padding: var(--space-md);
  background: var(--color-bg-card);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
}

.palette-title {
  font-size: var(--font-size-xs);
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--color-text-secondary);
}

.field-row {
  display: flex;
  gap: var(--space-sm);
  align-items: center;
}

.palette-input {
  flex: 1;
  padding: var(--space-sm);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  font-size: var(--font-size-sm);
  font-family: inherit;
  background: var(--color-bg);
  color: var(--color-text);
  outline: none;
}

.palette-input:focus {
  border-color: var(--color-primary);
}

.palette-input--short {
  max-width: 80px;
}

.palette-select {
  padding: var(--space-sm);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  font-size: var(--font-size-xs);
  font-family: inherit;
  background: var(--color-bg);
  color: var(--color-text);
  outline: none;
  cursor: pointer;
}

.palette-add {
  padding: var(--space-sm);
  font-size: var(--font-size-xs);
  font-weight: 600;
  color: var(--color-primary);
  background: var(--color-primary-light);
  border-radius: var(--radius-sm);
  transition: all var(--transition-fast);
}

.palette-add:hover:not(:disabled) {
  background: var(--color-primary);
  color: white;
}

.palette-add:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.date-preview {
  font-size: var(--font-size-sm);
  font-weight: 500;
  flex: 1;
}

.upload-row {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
}

.upload-btn {
  padding: var(--space-sm) var(--space-md);
  font-size: var(--font-size-xs);
  font-weight: 600;
  color: var(--color-text-secondary);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  transition: all var(--transition-fast);
}

.upload-btn:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.upload-preview {
  width: 40px;
  height: 40px;
  object-fit: contain;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
}

.group-btn {
  padding: var(--space-sm) var(--space-md);
  font-size: var(--font-size-xs);
  font-weight: 700;
  border: 2px dashed var(--color-border);
  border-radius: var(--radius-md);
  color: var(--color-text-secondary);
  transition: all var(--transition-fast);
}

.group-btn:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.group-btn--active {
  background: var(--color-primary-light);
  border-color: var(--color-primary);
  border-style: solid;
  color: var(--color-primary);
}

/* Preview area */
.preview-area {
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
}

.preview-toolbar {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
}

.page-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: var(--radius-sm);
  color: var(--color-text-secondary);
  transition: all var(--transition-fast);
}

.page-btn:hover:not(:disabled) {
  background: var(--color-bg-hover);
  color: var(--color-text);
}

.page-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.page-indicator {
  font-size: var(--font-size-sm);
  font-weight: 600;
  color: var(--color-text-secondary);
}

.toolbar-btn {
  margin-left: auto;
  padding: var(--space-sm) var(--space-md);
  font-size: var(--font-size-xs);
  font-weight: 500;
  color: var(--color-text-secondary);
  border-radius: var(--radius-sm);
  transition: all var(--transition-fast);
}

.toolbar-btn:hover {
  background: var(--color-bg-hover);
  color: var(--color-text);
}

.preview-container {
  position: relative;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  overflow: hidden;
  background: #e0e0e0;
  touch-action: none;
  user-select: none;
}

.preview-page {
  display: block;
  width: 100%;
  height: auto;
  background: white;
}

.loading-state {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 400px;
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

/* Signature items on preview */
.sig-item {
  position: absolute;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: grab;
  border: 1.5px dashed var(--color-primary);
  border-radius: 3px;
  background: rgba(74, 108, 247, 0.06);
  transition: box-shadow var(--transition-fast);
}

.sig-item:hover {
  box-shadow: 0 0 0 2px rgba(74, 108, 247, 0.25);
}

.sig-item--dragging {
  cursor: grabbing;
  box-shadow: 0 0 0 2px rgba(74, 108, 247, 0.4);
  z-index: 10;
}

.sig-item--grouped {
  border-color: #e67e22;
  background: rgba(230, 126, 34, 0.06);
}

.sig-text {
  white-space: nowrap;
  font-weight: 500;
  color: #1a1a2e;
  pointer-events: none;
  padding: 0 4px;
}

.sig-img {
  width: 100%;
  height: 100%;
  object-fit: contain;
  pointer-events: none;
}

.sig-delete {
  position: absolute;
  top: -8px;
  right: -8px;
  width: 18px;
  height: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-danger);
  color: white;
  border-radius: 50%;
  opacity: 0;
  transition: opacity var(--transition-fast);
  cursor: pointer;
  z-index: 5;
}

.sig-item:hover .sig-delete {
  opacity: 1;
}

.actions {
  display: flex;
  justify-content: flex-end;
}

@media (max-width: 700px) {
  .sign-layout {
    grid-template-columns: 1fr;
  }

  .palette {
    max-height: none;
    flex-direction: row;
    flex-wrap: wrap;
  }

  .palette-section {
    flex: 1;
    min-width: 200px;
  }
}
</style>
