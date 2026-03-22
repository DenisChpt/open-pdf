<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

const props = defineProps<{
  label?: string;
  accept?: string[];
}>();

const emit = defineEmits<{
  select: [];
  drop: [paths: string[]];
}>();

const isDragging = ref(false);
let unlisten: (() => void) | null = null;

onMounted(async () => {
  const webview = getCurrentWebviewWindow();
  unlisten = await webview.onDragDropEvent((event) => {
    if (event.payload.type === "over") {
      isDragging.value = true;
    } else if (event.payload.type === "drop") {
      isDragging.value = false;
      const paths = filterByExtension(event.payload.paths);
      if (paths.length > 0) {
        emit("drop", paths);
      }
    } else {
      // cancelled
      isDragging.value = false;
    }
  });
});

onUnmounted(() => {
  unlisten?.();
});

function filterByExtension(paths: string[]): string[] {
  const allowed = props.accept ?? ["pdf"];
  return paths.filter((p) => {
    const ext = p.split(".").pop()?.toLowerCase() ?? "";
    return allowed.includes(ext);
  });
}

function onDragOver(event: DragEvent) {
  event.preventDefault();
}

function onWebDrop(event: DragEvent) {
  event.preventDefault();
  isDragging.value = false;
  // Tauri's onDragDropEvent handles the actual paths
}
</script>

<template>
  <div
    class="drop-zone"
    :class="{ 'drop-zone--active': isDragging }"
    @dragover="onDragOver"
    @drop="onWebDrop"
    @click="emit('select')"
  >
    <div class="drop-zone-content">
      <div class="drop-zone-icon">
        <svg
          width="48"
          height="48"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
          <polyline points="17 8 12 3 7 8" />
          <line x1="12" y1="3" x2="12" y2="15" />
        </svg>
      </div>
      <p class="drop-zone-label">{{ label ?? "Cliquez pour choisir vos fichiers PDF" }}</p>
      <p class="drop-zone-hint">ou glissez-les ici</p>
    </div>
  </div>
</template>

<style scoped>
.drop-zone {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: clamp(140px, 25vw, 200px);
  border: 2px dashed var(--color-border);
  border-radius: var(--radius-lg);
  background: var(--color-bg-card);
  cursor: pointer;
  transition:
    border-color var(--transition-normal),
    background-color var(--transition-normal);
  user-select: none;
}

.drop-zone:hover,
.drop-zone--active {
  border-color: var(--color-primary);
  background: var(--color-bg-drop);
}

.drop-zone-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-sm);
  padding: var(--space-xl);
}

.drop-zone-icon {
  color: var(--color-text-muted);
  transition: color var(--transition-normal);
}

.drop-zone:hover .drop-zone-icon,
.drop-zone--active .drop-zone-icon {
  color: var(--color-primary);
}

.drop-zone-label {
  font-size: var(--font-size-md);
  font-weight: 500;
  color: var(--color-text-secondary);
}

.drop-zone-hint {
  font-size: var(--font-size-sm);
  color: var(--color-text-muted);
}
</style>
