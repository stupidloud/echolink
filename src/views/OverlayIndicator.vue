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

let unlistenRecording = null
let unlistenLevel = null

onMounted(async () => {
  try {
    unlistenRecording = await listen('recording-state', (event) => {
      isRecording.value = event.payload
      // Start (and reset) flat; the audio-level feed drives motion from here.
      barHeights.value = event.payload ? [4, 4, 4, 4, 4] : [8, 8, 8, 8, 8]
    })
  } catch {}
  try {
    unlistenLevel = await listen('audio-level', (event) => {
      if (!isRecording.value) return
      // Noise gate: below the floor the bars stay flat, so silence shows no
      // motion even while Right Alt is held down.
      const level = event.payload < 0.04 ? 0 : event.payload
      barHeights.value = Array.from({ length: 5 }, (_, i) => {
        const factor = 1 + Math.sin(i * 1.2) * 0.3
        return Math.max(4, Math.min(24, 4 + level * 32 * factor))
      })
    })
  } catch {}
})

onUnmounted(() => {
  if (unlistenRecording) unlistenRecording()
  if (unlistenLevel) unlistenLevel()
})
</script>

<style>
* { margin: 0; padding: 0; box-sizing: border-box; }

body {
  background: transparent;
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
