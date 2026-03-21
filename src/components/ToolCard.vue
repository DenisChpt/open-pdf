<script setup lang="ts">
import type { Tool } from "../types/tools";

defineProps<{
  tool: Tool;
}>();
</script>

<template>
  <router-link :to="tool.route" class="tool-card">
    <div class="tool-icon" :style="{ backgroundColor: tool.color + '12', color: tool.color }">
      <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
        <path :d="tool.icon" />
      </svg>
    </div>
    <div class="tool-info">
      <h3 class="tool-name">{{ tool.name }}</h3>
      <p class="tool-description">{{ tool.description }}</p>
    </div>
    <div class="tool-arrow">
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <polyline points="9 18 15 12 9 6" />
      </svg>
    </div>
  </router-link>
</template>

<style scoped>
.tool-card {
  display: flex;
  align-items: center;
  gap: var(--space-lg);
  padding: var(--space-lg) var(--space-xl);
  background: var(--color-bg-card);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  text-decoration: none;
  color: var(--color-text);
  box-shadow: var(--shadow-card);
  transition:
    box-shadow var(--transition-smooth),
    border-color var(--transition-smooth),
    transform var(--transition-smooth);
}

.tool-card:hover {
  box-shadow: var(--shadow-md);
  border-color: var(--color-border-active);
  transform: translateY(-2px);
}

.tool-card:active {
  transform: translateY(0);
}

.tool-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 56px;
  height: 56px;
  border-radius: var(--radius-md);
  flex-shrink: 0;
  transition: transform var(--transition-smooth);
}

.tool-card:hover .tool-icon {
  transform: scale(1.08);
}

.tool-info {
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
  flex: 1;
}

.tool-name {
  font-size: var(--font-size-md);
  font-weight: 600;
  letter-spacing: -0.01em;
}

.tool-description {
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
  line-height: 1.4;
}

.tool-arrow {
  color: var(--color-text-muted);
  flex-shrink: 0;
  transition: all var(--transition-smooth);
}

.tool-card:hover .tool-arrow {
  color: var(--color-primary);
  transform: translateX(4px);
}

@media (max-width: 480px) {
  .tool-icon {
    width: 44px;
    height: 44px;
  }

  .tool-icon svg {
    width: 22px;
    height: 22px;
  }

  .tool-arrow {
    display: none;
  }
}
</style>
