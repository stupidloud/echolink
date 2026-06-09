<template>
  <div class="overlay" :class="{ recording: isRecording }">
    <div class="waveform">
      <span v-for="(h, i) in barHeights" :key="i" class="bar" :style="{ height: h + 'px' }"></span>
    </div>
    <div class="status-light"></div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import { listen } from '@tauri-apps/api/event'

const isRecording = ref(false)
const barHeights = ref([8, 8, 8, 8, 8])

onMounted(async () => {
  try {
    await listen('recording-state', (event) => {
      isRecording.value = event.payload
      if (!event.payload) barHeights.value = [8, 8, 8, 8, 8]
    })
  } catch {}
  try {
    await listen('audio-level', (event) => {
      if (!isRecording.value) return
      const level = event.payload
      const heights = [4, 4, 4, 4, 4]
      for (let i = 0; i < 5; i++) {
        const factor = 1 + Math.sin(i * 1.2) * 0.3
        heights[i] = Math.max(4, Math.min(24, 4 + level * 32 * factor))
      }
      barHeights.value = heights
    })
  } catch {}
})
</script>

<style>
* { margin: 0; padding: 0; box-sizing: border-box; }

body {
  background: #1A1A1A;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100vh;
}

.overlay {
  display: flex;
  align-items: center;
  gap: 8px;
  background: rgba(26, 26, 26, 0.85);
  padding: 8px 16px;
  border-radius: 20px;
}

.waveform {
  display: flex;
  align-items: center;
  gap: 3px;
  height: 24px;
}

.bar {
  display: inline-block;
  width: 4px;
  height: 8px;
  background: #C8B496;
  border-radius: 2px;
  transition: height 0.12s ease-out;
}

.recording .bar {
  background: #EF4444;
}

.status-light {
  width: 10px;
  height: 10px;
  background: #C8B496;
  border-radius: 50%;
  transition: background 0.3s;
}

.recording .status-light {
  background: #EF4444;
  animation: pulse 1s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}
</style>
