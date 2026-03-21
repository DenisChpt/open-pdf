<script setup lang="ts">
defineProps<{
  label: string;
  variant?: "primary" | "secondary" | "danger";
  disabled?: boolean;
  loading?: boolean;
}>();

defineEmits<{
  click: [];
}>();
</script>

<template>
  <button
    class="action-button"
    :class="[
      `action-button--${variant ?? 'primary'}`,
      { 'action-button--loading': loading },
    ]"
    :disabled="disabled || loading"
    @click="$emit('click')"
  >
    <span v-if="loading" class="spinner" />
    <span :class="{ invisible: loading }">{{ label }}</span>
  </button>
</template>

<style scoped>
.action-button {
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-sm);
  padding: var(--space-sm) var(--space-lg);
  font-weight: 600;
  font-size: var(--font-size-sm);
  border-radius: var(--radius-md);
  transition:
    background-color var(--transition-fast),
    transform var(--transition-fast),
    box-shadow var(--transition-fast);
}

.action-button:active:not(:disabled) {
  transform: scale(0.98);
}

.action-button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.action-button--primary {
  background: var(--color-primary);
  color: white;
}

.action-button--primary:hover:not(:disabled) {
  background: var(--color-primary-hover);
  box-shadow: var(--shadow-md);
}

.action-button--secondary {
  background: var(--color-bg);
  color: var(--color-text);
  border: 1px solid var(--color-border);
}

.action-button--secondary:hover:not(:disabled) {
  background: var(--color-bg-hover);
}

.action-button--danger {
  background: var(--color-danger);
  color: white;
}

.action-button--danger:hover:not(:disabled) {
  background: var(--color-danger-hover);
}

.spinner {
  position: absolute;
  width: 20px;
  height: 20px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top-color: white;
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}

.invisible {
  visibility: hidden;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
