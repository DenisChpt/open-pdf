<script setup lang="ts">
import ToolCard from "../components/ToolCard.vue";
import { TOOLS } from "../types/tools";
</script>

<template>
  <div class="home">
    <section class="hero">
      <div class="hero-badge">100% local &middot; vos fichiers restent prives</div>
      <h1 class="hero-title">Tous vos outils PDF<br />en un seul endroit</h1>
      <p class="hero-subtitle">
        Fusionnez, divisez, compressez et reorganisez vos fichiers PDF
        directement sur votre ordinateur.
      </p>
    </section>

    <section class="tools-grid">
      <ToolCard v-for="tool in TOOLS" :key="tool.id" :tool="tool" />
    </section>
  </div>
</template>

<style scoped>
.home {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: var(--space-2xl) var(--page-padding);
  gap: var(--space-2xl);
  max-width: 720px;
  margin: 0 auto;
  width: 100%;
}

.hero {
  text-align: center;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-md);
}

.hero-badge {
  display: inline-flex;
  padding: var(--space-xs) var(--space-md);
  background: var(--color-success-light);
  color: var(--color-success);
  font-size: var(--font-size-xs);
  font-weight: 600;
  border-radius: 100px;
  letter-spacing: 0.02em;
  text-transform: uppercase;
}

.hero-title {
  font-size: var(--font-size-3xl);
  font-weight: 800;
  line-height: 1.15;
  letter-spacing: -0.03em;
  color: var(--color-text);
}

.hero-subtitle {
  font-size: var(--font-size-lg);
  color: var(--color-text-secondary);
  max-width: 460px;
  line-height: 1.6;
}

.tools-grid {
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
  width: 100%;
}
</style>
