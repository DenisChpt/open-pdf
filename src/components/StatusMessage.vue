<script setup lang="ts">
defineProps<{
  type: "success" | "error" | "info";
  message: string;
}>();
</script>

<template>
  <div class="status" :class="`status--${type}`">
    <span class="status-icon">
      <template v-if="type === 'success'">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="20 6 9 17 4 12" />
        </svg>
      </template>
      <template v-else-if="type === 'error'">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="10" />
          <line x1="15" y1="9" x2="9" y2="15" />
          <line x1="9" y1="9" x2="15" y2="15" />
        </svg>
      </template>
      <template v-else>
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="10" />
          <line x1="12" y1="16" x2="12" y2="12" />
          <line x1="12" y1="8" x2="12.01" y2="8" />
        </svg>
      </template>
    </span>
    <span class="status-text">{{ message }}</span>
  </div>
</template>

<style scoped>
.status {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  padding: var(--space-md);
  border-radius: var(--radius-md);
  font-size: var(--font-size-sm);
  font-weight: 500;
}

.status--success {
  background: #e8f5e9;
  color: #2e7d32;
}

.status--error {
  background: #fce4ec;
  color: #c62828;
}

.status--info {
  background: var(--color-primary-light);
  color: var(--color-primary);
}

.status-icon {
  display: flex;
  flex-shrink: 0;
}

@media (prefers-color-scheme: dark) {
  .status--success {
    background: #1b5e2033;
    color: #66bb6a;
  }
  .status--error {
    background: #b7121233;
    color: #ef5350;
  }
  .status--info {
    background: var(--color-primary-light);
    color: var(--color-primary);
  }
}
</style>
