<script setup lang="ts">
import { ref, computed } from "vue";
import DropZone from "../components/DropZone.vue";
import ActionButton from "../components/ActionButton.vue";
import StatusMessage from "../components/StatusMessage.vue";
import { usePdfApi } from "../composables/usePdfApi";
import { useFileDialog } from "../composables/useFileDialog";
import type { SelectedFile } from "../types/pdf";

const { getPdfInfo, compressPdf } = usePdfApi();
const { pickPdfFiles, pickSavePath } = useFileDialog();

const file = ref<SelectedFile | null>(null);
const isLoading = ref(false);
const result = ref<{ original: number; compressed: number } | null>(null);
const errorMessage = ref<string | null>(null);

const reduction = computed(() => {
  if (!result.value) return 0;
  return Math.round(
    ((result.value.original - result.value.compressed) / result.value.original) * 100,
  );
});

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} o`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} Ko`;
  return `${(bytes / (1024 * 1024)).toFixed(2)} Mo`;
}

async function loadPaths(paths: string[]) {
  if (paths.length === 0) return;
  errorMessage.value = null;
  result.value = null;
  try {
    const info = await getPdfInfo(paths[0]);
    file.value = {
      path: info.path,
      name: info.file_name,
      pageCount: info.page_count,
      sizeBytes: info.file_size_bytes,
    };
  } catch (error) {
    errorMessage.value = `Erreur lors de la lecture : ${error}`;
  }
}

async function selectFile() {
  const paths = await pickPdfFiles();
  await loadPaths(paths);
}

async function compress() {
  if (!file.value || isLoading.value) return;
  isLoading.value = true;
  errorMessage.value = null;
  result.value = null;

  try {
    const outputPath = await pickSavePath(`${file.value.name}_compressed.pdf`);
    if (!outputPath) {
      isLoading.value = false;
      return;
    }

    const res = await compressPdf({
      input_path: file.value.path,
      output_path: outputPath,
    });

    result.value = {
      original: res.original_size,
      compressed: res.compressed_size,
    };
  } catch (error) {
    errorMessage.value = `Erreur lors de la compression : ${error}`;
  } finally {
    isLoading.value = false;
  }
}

function reset() {
  file.value = null;
  result.value = null;
  errorMessage.value = null;
}
</script>

<template>
  <div class="compress-view">
    <div class="view-header">
      <h1 class="view-title">Compresser PDF</h1>
      <p class="view-subtitle">
        Reduisez la taille de vos fichiers PDF en optimisant leur contenu.
      </p>
    </div>

    <DropZone
      v-if="!file"
      label="Cliquez pour choisir un fichier PDF"
      @select="selectFile"
      @drop="loadPaths"
    />

    <template v-else>
      <div class="file-card">
        <div class="file-card-icon">
          <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
            <polyline points="14 2 14 8 20 8" />
          </svg>
        </div>
        <div class="file-card-info">
          <p class="file-card-name">{{ file.name }}.pdf</p>
          <p class="file-card-meta">
            {{ file.pageCount }} page{{ file.pageCount > 1 ? "s" : "" }}
            &middot; {{ formatSize(file.sizeBytes) }}
          </p>
        </div>
        <button class="change-btn" @click="reset">Changer</button>
      </div>

      <!-- Compression result -->
      <div v-if="result" class="result-card">
        <div class="result-header">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="20 6 9 17 4 12" />
          </svg>
          <span>Compression terminee</span>
        </div>
        <div class="result-stats">
          <div class="stat">
            <span class="stat-label">Avant</span>
            <span class="stat-value">{{ formatSize(result.original) }}</span>
          </div>
          <div class="stat-arrow">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <line x1="5" y1="12" x2="19" y2="12" />
              <polyline points="12 5 19 12 12 19" />
            </svg>
          </div>
          <div class="stat">
            <span class="stat-label">Apres</span>
            <span class="stat-value stat-value--highlight">{{ formatSize(result.compressed) }}</span>
          </div>
          <div class="stat stat--badge">
            <span class="reduction-badge" :class="{ 'reduction-badge--zero': reduction <= 0 }">
              {{ reduction > 0 ? `-${reduction}%` : "aucune reduction" }}
            </span>
          </div>
        </div>
      </div>

      <div v-if="!result" class="actions">
        <ActionButton
          label="Compresser"
          :loading="isLoading"
          @click="compress"
        />
      </div>
    </template>

    <StatusMessage
      v-if="errorMessage"
      type="error"
      :message="errorMessage"
    />
  </div>
</template>

<style scoped>
.compress-view {
  display: flex;
  flex-direction: column;
  gap: var(--space-lg);
  padding: var(--space-2xl) var(--page-padding);
  max-width: 600px;
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

.file-card {
  display: flex;
  align-items: center;
  gap: var(--space-md);
  padding: var(--space-lg);
  background: var(--color-bg-card);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-card);
}

.file-card-icon {
  color: var(--color-primary);
  flex-shrink: 0;
}

.file-card-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.file-card-name {
  font-weight: 600;
  font-size: var(--font-size-sm);
}

.file-card-meta {
  font-size: var(--font-size-xs);
  color: var(--color-text-secondary);
}

.change-btn {
  padding: var(--space-sm) var(--space-md);
  color: var(--color-text-secondary);
  border-radius: var(--radius-sm);
  font-size: var(--font-size-sm);
  font-weight: 500;
  transition: all var(--transition-fast);
}

.change-btn:hover {
  background: var(--color-bg-hover);
  color: var(--color-text);
}

.result-card {
  display: flex;
  flex-direction: column;
  gap: var(--space-lg);
  padding: var(--space-lg);
  background: var(--color-success-light);
  border: 1px solid var(--color-success);
  border-radius: var(--radius-lg);
}

.result-header {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  color: var(--color-success);
  font-weight: 600;
}

.result-stats {
  display: flex;
  align-items: center;
  gap: var(--space-lg);
}

.stat {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.stat--badge {
  margin-left: auto;
}

.stat-label {
  font-size: var(--font-size-xs);
  color: var(--color-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  font-weight: 500;
}

.stat-value {
  font-size: var(--font-size-lg);
  font-weight: 700;
  color: var(--color-text);
}

.stat-value--highlight {
  color: var(--color-success);
}

.stat-arrow {
  color: var(--color-text-muted);
}

.reduction-badge {
  display: inline-flex;
  padding: var(--space-xs) var(--space-md);
  background: var(--color-success);
  color: white;
  font-size: var(--font-size-sm);
  font-weight: 700;
  border-radius: 100px;
}

.reduction-badge--zero {
  background: var(--color-text-muted);
}

.actions {
  display: flex;
  justify-content: flex-end;
}
</style>
