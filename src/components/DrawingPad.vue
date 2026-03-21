<script setup lang="ts">
import { ref, onMounted } from "vue";

const emit = defineEmits<{
  drawn: [dataUrl: string];
}>();

const canvasRef = ref<HTMLCanvasElement>();
let isDrawing = false;

onMounted(() => {
  const canvas = canvasRef.value;
  if (!canvas) return;
  const ctx = canvas.getContext("2d");
  if (!ctx) return;
  ctx.lineCap = "round";
  ctx.lineJoin = "round";
  ctx.lineWidth = 2;
  ctx.strokeStyle = "#1a1a2e";
});

function getPos(e: PointerEvent) {
  const canvas = canvasRef.value!;
  const rect = canvas.getBoundingClientRect();
  return {
    x: ((e.clientX - rect.left) / rect.width) * canvas.width,
    y: ((e.clientY - rect.top) / rect.height) * canvas.height,
  };
}

function onPointerDown(e: PointerEvent) {
  isDrawing = true;
  const ctx = canvasRef.value?.getContext("2d");
  if (!ctx) return;
  const pos = getPos(e);
  ctx.beginPath();
  ctx.moveTo(pos.x, pos.y);
  (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
}

function onPointerMove(e: PointerEvent) {
  if (!isDrawing) return;
  const ctx = canvasRef.value?.getContext("2d");
  if (!ctx) return;
  const pos = getPos(e);
  ctx.lineTo(pos.x, pos.y);
  ctx.stroke();
}

function onPointerUp() {
  if (!isDrawing) return;
  isDrawing = false;
  if (canvasRef.value) {
    emit("drawn", canvasRef.value.toDataURL("image/png"));
  }
}

function clear() {
  const canvas = canvasRef.value;
  if (!canvas) return;
  const ctx = canvas.getContext("2d");
  if (!ctx) return;
  ctx.clearRect(0, 0, canvas.width, canvas.height);
}

defineExpose({ clear });
</script>

<template>
  <div class="drawing-pad">
    <canvas
      ref="canvasRef"
      width="400"
      height="150"
      class="drawing-canvas"
      @pointerdown="onPointerDown"
      @pointermove="onPointerMove"
      @pointerup="onPointerUp"
      @pointerleave="onPointerUp"
    />
    <button class="clear-btn" @click="clear">Effacer</button>
  </div>
</template>

<style scoped>
.drawing-pad {
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
}

.drawing-canvas {
  width: 100%;
  height: auto;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background: white;
  cursor: crosshair;
  touch-action: none;
}

.clear-btn {
  align-self: flex-end;
  padding: var(--space-xs) var(--space-md);
  font-size: var(--font-size-xs);
  font-weight: 600;
  color: var(--color-text-secondary);
  border-radius: var(--radius-sm);
  transition: all var(--transition-fast);
}

.clear-btn:hover {
  background: var(--color-bg-hover);
  color: var(--color-danger);
}
</style>
