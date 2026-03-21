<script setup lang="ts">
import { ref, computed } from "vue";
import { VueDraggable } from "vue-draggable-plus";
import DropZone from "../components/DropZone.vue";
import ActionButton from "../components/ActionButton.vue";
import StatusMessage from "../components/StatusMessage.vue";
import { usePdfApi } from "../composables/usePdfApi";
import { usePdfRenderer } from "../composables/usePdfRenderer";
import { useFileDialog } from "../composables/useFileDialog";

interface MergeFileItem {
  id: string;
  path: string;
  name: string;
  pageCount: number;
  sizeBytes: number;
  thumbnailUrl: string | null;
}

const { getPdfInfo, getPdfData, mergePdfs } = usePdfApi();
const { renderThumbnails } = usePdfRenderer();
const { pickPdfFiles, pickSavePath } = useFileDialog();

const files = ref<MergeFileItem[]>([]);
const isLoading = ref(false);
const isAdding = ref(false);
const statusMessage = ref<{ type: "success" | "error"; message: string } | null>(null);

const canMerge = computed(() => files.value.length >= 2 && !isLoading.value);

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} o`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} Ko`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} Mo`;
}

async function addFiles() {
  if (isAdding.value) return;
  isAdding.value = true;
  statusMessage.value = null;

  try {
    const paths = await pickPdfFiles();
    for (const path of paths) {
      if (files.value.some((f) => f.path === path)) continue;

      const info = await getPdfInfo(path);
      const item: MergeFileItem = {
        id: `file-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
        path: info.path,
        name: info.file_name,
        pageCount: info.page_count,
        sizeBytes: info.file_size_bytes,
        thumbnailUrl: null,
      };
      files.value.push(item);

      // Render first page thumbnail in background
      loadThumbnail(item);
    }
  } catch (error) {
    statusMessage.value = {
      type: "error",
      message: `Erreur lors de l'ajout : ${error}`,
    };
  } finally {
    isAdding.value = false;
  }
}

async function loadThumbnail(item: MergeFileItem) {
  try {
    const base64Data = await getPdfData(item.path);
    const thumbnails = await renderThumbnails(base64Data, 1);
    // Find the item in the array (it may have moved due to drag)
    const found = files.value.find((f) => f.id === item.id);
    if (found && thumbnails[0]) {
      found.thumbnailUrl = thumbnails[0];
    }
  } catch {
    // Thumbnail loading is non-critical, silently ignore
  }
}

function removeFile(index: number) {
  files.value.splice(index, 1);
  statusMessage.value = null;
}

async function merge() {
  if (!canMerge.value) return;
  isLoading.value = true;
  statusMessage.value = null;

  try {
    const outputPath = await pickSavePath("merged.pdf");
    if (!outputPath) {
      isLoading.value = false;
      return;
    }

    const result = await mergePdfs({
      input_paths: files.value.map((f) => f.path),
      output_path: outputPath,
    });

    const sizeMb = (result.file_size_bytes / (1024 * 1024)).toFixed(1);
    statusMessage.value = {
      type: "success",
      message: `PDF fusionne avec succes (${sizeMb} Mo)`,
    };
  } catch (error) {
    statusMessage.value = {
      type: "error",
      message: `Erreur lors de la fusion : ${error}`,
    };
  } finally {
    isLoading.value = false;
  }
}
</script>

<template>
  <div class="merge-view">
    <div class="view-header">
      <h1 class="view-title">Fusionner PDF</h1>
      <p class="view-subtitle">
        Glissez les fichiers pour changer l'ordre de fusion.
        La premiere page de chaque PDF est affichee en apercu.
      </p>
    </div>

    <DropZone
      v-if="files.length === 0"
      label="Cliquez pour choisir vos fichiers PDF"
      @select="addFiles"
    />

    <template v-else>
      <VueDraggable
        v-model="files"
        :animation="250"
        ghost-class="file-ghost"
        drag-class="file-drag"
        class="files-grid"
      >
        <div
          v-for="(file, index) in files"
          :key="file.id"
          class="file-card"
        >
          <div class="file-thumbnail">
            <img
              v-if="file.thumbnailUrl"
              :src="file.thumbnailUrl"
              :alt="file.name"
              draggable="false"
            />
            <div v-else class="file-placeholder">
              <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
                <polyline points="14 2 14 8 20 8" />
              </svg>
            </div>
            <button
              class="file-delete"
              title="Retirer ce fichier"
              @click.stop="removeFile(index)"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                <line x1="18" y1="6" x2="6" y2="18" />
                <line x1="6" y1="6" x2="18" y2="18" />
              </svg>
            </button>
            <span class="file-order">{{ index + 1 }}</span>
          </div>
          <div class="file-info">
            <span class="file-name" :title="file.name">{{ file.name }}</span>
            <span class="file-meta">{{ file.pageCount }}p &middot; {{ formatSize(file.sizeBytes) }}</span>
          </div>
        </div>
      </VueDraggable>

      <div class="actions">
        <ActionButton
          label="Ajouter des fichiers"
          variant="secondary"
          :loading="isAdding"
          @click="addFiles"
        />
        <ActionButton
          label="Fusionner"
          :disabled="!canMerge"
          :loading="isLoading"
          @click="merge"
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
.merge-view {
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

/* Files Grid */
.files-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(var(--grid-min-thumb), 1fr));
  gap: var(--space-md);
}

.file-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-sm);
  cursor: grab;
  user-select: none;
}

.file-card:active {
  cursor: grabbing;
}

.file-thumbnail {
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

.file-thumbnail:hover {
  border-color: var(--color-primary);
  box-shadow: var(--shadow-md);
  transform: translateY(-2px);
}

.file-thumbnail img {
  width: 100%;
  height: 100%;
  object-fit: contain;
  background: white;
}

.file-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-bg);
  color: var(--color-text-muted);
}

.file-delete {
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
}

.file-delete:hover {
  background: var(--color-danger);
}

.file-thumbnail:hover .file-delete {
  opacity: 1;
}

.file-order {
  position: absolute;
  bottom: var(--space-xs);
  left: var(--space-xs);
  min-width: 22px;
  height: 22px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-primary);
  color: white;
  font-size: var(--font-size-xs);
  font-weight: 700;
  border-radius: 100px;
  padding: 0 var(--space-xs);
}

.file-info {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1px;
  width: 100%;
  text-align: center;
}

.file-name {
  font-size: var(--font-size-xs);
  font-weight: 600;
  color: var(--color-text);
  max-width: 100%;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-meta {
  font-size: 11px;
  color: var(--color-text-muted);
}

/* Drag states */
.file-ghost .file-thumbnail {
  border-color: var(--color-primary);
  box-shadow: var(--shadow-lg);
  opacity: 0.4;
}

.file-drag .file-thumbnail {
  border-color: var(--color-primary);
  box-shadow: var(--shadow-lg);
  transform: rotate(2deg) scale(1.05);
}

.actions {
  display: flex;
  gap: var(--space-md);
  justify-content: flex-end;
}
</style>
