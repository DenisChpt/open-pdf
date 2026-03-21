<script setup lang="ts">
import { ref, computed } from "vue";
import { VueDraggable } from "vue-draggable-plus";
import DropZone from "../components/DropZone.vue";
import ActionButton from "../components/ActionButton.vue";
import StatusMessage from "../components/StatusMessage.vue";
import { usePdfApi } from "../composables/usePdfApi";
import { usePdfRenderer } from "../composables/usePdfRenderer";
import { useFileDialog } from "../composables/useFileDialog";
import type { PageItem, SelectedFile } from "../types/pdf";

const { getPdfInfo, getPdfData, reorderPages } = usePdfApi();
const { renderThumbnails, isRendering } = usePdfRenderer();
const { pickPdfFiles, pickSavePath } = useFileDialog();

const file = ref<SelectedFile | null>(null);
const pages = ref<PageItem[]>([]);
const selected = ref<Set<string>>(new Set());
const isLoading = ref(false);
const statusMessage = ref<{ type: "success" | "error"; message: string } | null>(null);

const allSelected = computed(
  () => pages.value.length > 0 && selected.value.size === pages.value.length,
);

function toggleSelect(id: string) {
  const s = new Set(selected.value);
  if (s.has(id)) s.delete(id);
  else s.add(id);
  selected.value = s;
}

function toggleSelectAll() {
  if (allSelected.value) {
    selected.value = new Set();
  } else {
    selected.value = new Set(pages.value.map((p) => p.id));
  }
}

function rotateSelected(angle: number) {
  for (const page of pages.value) {
    if (selected.value.has(page.id)) {
      page.rotation = (page.rotation + angle + 360) % 360;
    }
  }
}

const hasChanges = computed(() => {
  if (!file.value) return false;
  const orderChanged =
    pages.value.some((p, i) => p.pageNumber !== i + 1) ||
    pages.value.length !== file.value.pageCount;
  const rotationChanged = pages.value.some((p) => p.rotation !== 0);
  return orderChanged || rotationChanged;
});

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

    pages.value = Array.from({ length: info.page_count }, (_, i) => ({
      id: `page-${i + 1}`,
      pageNumber: i + 1,
      thumbnailUrl: null,
      rotation: 0,
    }));

    const base64Data = await getPdfData(paths[0]);
    const thumbs = await renderThumbnails(base64Data, info.page_count);
    for (let i = 0; i < thumbs.length; i++) {
      if (pages.value[i]) {
        pages.value[i].thumbnailUrl = thumbs[i];
      }
    }
  } catch (error) {
    statusMessage.value = {
      type: "error",
      message: `Erreur lors de la lecture : ${error}`,
    };
  }
}

function removePage(index: number) {
  pages.value.splice(index, 1);
}

function rotateLeft(index: number) {
  pages.value[index].rotation = (pages.value[index].rotation + 270) % 360;
}

function rotateRight(index: number) {
  pages.value[index].rotation = (pages.value[index].rotation + 90) % 360;
}

async function save() {
  if (!file.value || isLoading.value) return;
  isLoading.value = true;
  statusMessage.value = null;

  try {
    const outputPath = await pickSavePath(`${file.value.name}_organized.pdf`);
    if (!outputPath) {
      isLoading.value = false;
      return;
    }

    // Build rotations map: only include pages that have been rotated
    const rotations: Record<number, number> = {};
    for (const page of pages.value) {
      if (page.rotation !== 0) {
        rotations[page.pageNumber] = page.rotation;
      }
    }

    await reorderPages({
      input_path: file.value.path,
      new_order: pages.value.map((p) => p.pageNumber),
      rotations,
      output_path: outputPath,
    });

    statusMessage.value = {
      type: "success",
      message: "PDF reorganise avec succes",
    };
  } catch (error) {
    statusMessage.value = {
      type: "error",
      message: `Erreur lors de la reorganisation : ${error}`,
    };
  } finally {
    isLoading.value = false;
  }
}

function reset() {
  file.value = null;
  pages.value = [];
  selected.value = new Set();
  statusMessage.value = null;
}
</script>

<template>
  <div class="organize-view">
    <div class="view-header">
      <h1 class="view-title">Organiser PDF</h1>
      <p class="view-subtitle">
        Glissez les pages pour les reorganiser, faites-les pivoter ou supprimez-les.
      </p>
    </div>

    <DropZone
      v-if="!file"
      label="Cliquez pour choisir un fichier PDF"
      @select="selectFile"
    />

    <template v-else>
      <div class="toolbar">
        <div class="toolbar-info">
          <span class="toolbar-filename">{{ file.name }}.pdf</span>
          <span class="toolbar-pages">{{ pages.length }} page{{ pages.length > 1 ? "s" : "" }}</span>
        </div>
        <div class="toolbar-actions">
          <button
            class="toolbar-btn toolbar-btn--select"
            :class="{ 'toolbar-btn--active': allSelected }"
            @click="toggleSelectAll"
          >
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <polyline v-if="allSelected" points="9 11 12 14 22 4" />
              <rect x="3" y="3" width="18" height="18" rx="3" />
            </svg>
            {{ allSelected ? "Deselectionner" : "Tout selectionner" }}
          </button>

          <template v-if="selected.size > 0">
            <span class="toolbar-divider" />
            <span class="toolbar-selection">{{ selected.size }} sel.</span>
            <button class="toolbar-icon-btn" title="Pivoter la selection a gauche" @click="rotateSelected(270)">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <polyline points="1 4 1 10 7 10" />
                <path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10" />
              </svg>
            </button>
            <button class="toolbar-icon-btn" title="Pivoter la selection a droite" @click="rotateSelected(90)">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <polyline points="23 4 23 10 17 10" />
                <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10" />
              </svg>
            </button>
          </template>

          <span class="toolbar-divider" />
          <button class="toolbar-btn" @click="reset">Changer</button>
        </div>
      </div>

      <div v-if="isRendering" class="loading-state">
        <div class="loading-spinner" />
        <p>Chargement des aperçus...</p>
      </div>

      <VueDraggable
        v-model="pages"
        :animation="250"
        ghost-class="page-ghost"
        drag-class="page-drag"
        handle=".page-thumbnail"
        class="pages-grid"
      >
        <div
          v-for="(page, index) in pages"
          :key="page.id"
          class="page-card"
          :class="{ 'page-card--selected': selected.has(page.id) }"
        >
          <div
            class="page-thumbnail"
            :class="{ 'page-thumbnail--landscape': page.rotation === 90 || page.rotation === 270 }"
          >
            <!-- Selection checkbox -->
            <button
              class="page-select"
              :class="{ 'page-select--checked': selected.has(page.id) }"
              title="Selectionner"
              @click.stop="toggleSelect(page.id)"
            >
              <svg v-if="selected.has(page.id)" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
                <polyline points="20 6 9 17 4 12" />
              </svg>
            </button>
            <img
              v-if="page.thumbnailUrl"
              :src="page.thumbnailUrl"
              :alt="`Page ${page.pageNumber}`"
              draggable="false"
              class="page-img"
              :style="{ transform: `rotate(${page.rotation}deg)` }"
            />
            <div v-else class="page-placeholder" :style="{ transform: `rotate(${page.rotation}deg)` }">
              <span>{{ page.pageNumber }}</span>
            </div>
            <button
              class="page-delete"
              title="Supprimer cette page"
              @click.stop="removePage(index)"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                <line x1="18" y1="6" x2="6" y2="18" />
                <line x1="6" y1="6" x2="18" y2="18" />
              </svg>
            </button>
          </div>

          <!-- Rotation controls + label -->
          <div class="page-footer">
            <button
              class="rotate-btn"
              title="Pivoter a gauche"
              @click.stop="rotateLeft(index)"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <polyline points="1 4 1 10 7 10" />
                <path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10" />
              </svg>
            </button>
            <span class="page-label">{{ page.pageNumber }}</span>
            <button
              class="rotate-btn"
              title="Pivoter a droite"
              @click.stop="rotateRight(index)"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <polyline points="23 4 23 10 17 10" />
                <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10" />
              </svg>
            </button>
          </div>
        </div>
      </VueDraggable>

      <div class="actions">
        <ActionButton
          label="Enregistrer"
          :disabled="!hasChanges"
          :loading="isLoading"
          @click="save"
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
.organize-view {
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
  gap: var(--space-md);
}

.toolbar-filename {
  font-weight: 600;
  font-size: var(--font-size-sm);
}

.toolbar-pages {
  font-size: var(--font-size-xs);
  color: var(--color-text-secondary);
  padding: 2px var(--space-sm);
  background: var(--color-bg);
  border-radius: 100px;
}

.toolbar-actions {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
}

.toolbar-btn {
  padding: var(--space-sm) var(--space-md);
  color: var(--color-text-secondary);
  font-size: var(--font-size-xs);
  font-weight: 500;
  border-radius: var(--radius-sm);
  transition: all var(--transition-fast);
}

.toolbar-btn:hover {
  background: var(--color-bg-hover);
  color: var(--color-text);
}

.toolbar-btn--select {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
}

.toolbar-btn--active {
  color: var(--color-primary);
}

.toolbar-divider {
  width: 1px;
  height: 20px;
  background: var(--color-border);
  flex-shrink: 0;
}

.toolbar-selection {
  font-size: var(--font-size-xs);
  font-weight: 600;
  color: var(--color-primary);
  white-space: nowrap;
}

.toolbar-icon-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: var(--radius-sm);
  color: var(--color-text-secondary);
  transition: all var(--transition-fast);
}

.toolbar-icon-btn:hover {
  background: var(--color-primary-light);
  color: var(--color-primary);
}

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

/* Pages Grid */
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
  cursor: grab;
  transition:
    border-color var(--transition-smooth),
    box-shadow var(--transition-smooth),
    transform var(--transition-smooth),
    aspect-ratio var(--transition-smooth);
}

.page-thumbnail--landscape {
  aspect-ratio: 297 / 210;
}

.page-thumbnail:hover {
  border-color: var(--color-primary);
  box-shadow: var(--shadow-md);
  transform: translateY(-2px);
}

.page-thumbnail:active {
  cursor: grabbing;
}

.page-img {
  width: 100%;
  height: 100%;
  object-fit: contain;
  background: white;
  transition: transform var(--transition-smooth);
}

.page-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-bg);
  font-size: var(--font-size-xl);
  font-weight: 700;
  color: var(--color-text-muted);
  transition: transform var(--transition-smooth);
}

/* Selection checkbox */
.page-select {
  position: absolute;
  top: var(--space-xs);
  left: var(--space-xs);
  width: 22px;
  height: 22px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 2px solid rgba(255, 255, 255, 0.7);
  border-radius: var(--radius-sm);
  background: rgba(0, 0, 0, 0.25);
  color: white;
  cursor: pointer;
  opacity: 0;
  transition: all var(--transition-fast);
  z-index: 2;
}

.page-thumbnail:hover .page-select,
.page-select--checked {
  opacity: 1;
}

.page-select--checked {
  background: var(--color-primary);
  border-color: var(--color-primary);
}

.page-card--selected .page-thumbnail {
  border-color: var(--color-primary);
  box-shadow: 0 0 0 2px var(--color-primary-light), var(--shadow-md);
}

.page-delete {
  position: absolute;
  top: var(--space-xs);
  right: var(--space-xs);
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.6);
  color: white;
  border-radius: 50%;
  opacity: 0;
  transition: opacity var(--transition-fast);
  cursor: pointer;
  z-index: 2;
}

.page-delete:hover {
  background: var(--color-danger);
}

.page-thumbnail:hover .page-delete {
  opacity: 1;
}

/* Footer: rotation buttons + page label */
.page-footer {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
}

.rotate-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  border-radius: var(--radius-sm);
  color: var(--color-text-muted);
  transition: all var(--transition-fast);
  cursor: pointer;
}

.rotate-btn:hover {
  background: var(--color-primary-light);
  color: var(--color-primary);
}

.page-label {
  font-size: var(--font-size-xs);
  font-weight: 600;
  color: var(--color-text-secondary);
  min-width: 16px;
  text-align: center;
}

/* Drag states */
.page-ghost .page-thumbnail {
  border-color: var(--color-primary);
  box-shadow: var(--shadow-lg);
  opacity: 0.4;
}

.page-drag .page-thumbnail {
  border-color: var(--color-primary);
  box-shadow: var(--shadow-lg);
  transform: rotate(2deg) scale(1.05);
}

.actions {
  display: flex;
  justify-content: flex-end;
}
</style>
