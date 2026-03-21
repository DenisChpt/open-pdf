<script setup lang="ts">
import { ref } from "vue";

defineProps<{
  label?: string;
  accept?: string;
}>();

const emit = defineEmits<{
  select: [];
}>();

const isDragging = ref(false);

function onDragEnter(event: DragEvent) {
  event.preventDefault();
  isDragging.value = true;
}

function onDragLeave(event: DragEvent) {
  event.preventDefault();
  const target = event.currentTarget as HTMLElement;
  const related = event.relatedTarget as Node | null;
  if (target && !target.contains(related)) {
    isDragging.value = false;
  }
}

function onDragOver(event: DragEvent) {
  event.preventDefault();
}

function onDrop(event: DragEvent) {
  event.preventDefault();
  isDragging.value = false;
  // Tauri handles file paths via dialog, drop is visual feedback only
  emit("select");
}
</script>

<template>
  <div
    class="drop-zone"
    :class="{ 'drop-zone--active': isDragging }"
    @dragenter="onDragEnter"
    @dragleave="onDragLeave"
    @dragover="onDragOver"
    @drop="onDrop"
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
