<script setup lang="ts">
import AppHeader from "./components/AppHeader.vue";
</script>

<template>
  <AppHeader />
  <main class="main-content">
    <router-view v-slot="{ Component }">
      <transition name="page" mode="out-in">
        <component :is="Component" />
      </transition>
    </router-view>
  </main>
</template>

<style scoped>
.main-content {
  flex: 1;
  overflow-y: auto;
}
</style>
