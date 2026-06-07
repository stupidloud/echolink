<template>
  <div class="dashboard">
    <!-- Header -->
    <div class="header">
      <h1 class="header-title">自然说话，完美书写 – 在任何应用中</h1>
      <div class="badge">
        <span>核心操作：按住 [Right Alt] 键开始录音，松开停止并转换</span>
      </div>
    </div>

    <!-- Stats Grid -->
    <div class="stats-grid">
      <div class="stats-row">
        <div class="stat-card">
          <div class="chart-placeholder"></div>
          <p class="card-title">整体个性化 0%</p>
          <a href="#" class="card-link">查看报告</a>
        </div>
        <div class="stat-card">
          <p class="card-value">{{ totalMinutes }} min</p>
          <p class="card-label">总口述时间</p>
        </div>
      </div>
      <div class="stats-row">
        <div class="stat-card">
          <p class="card-value">{{ totalChars }} 字</p>
          <p class="card-label">口述字数</p>
        </div>
        <div class="stat-card">
          <p class="card-value">{{ avgSpeed }} 字/分钟</p>
          <p class="card-label">平均口述速度</p>
        </div>
      </div>
    </div>

    <!-- Transcript Box -->
    <div class="transcript-box">
      <label class="transcript-label">最近一次转录结果</label>
      <p class="transcript-text">{{ transcript || '等待语音输入...' }}</p>
      <div v-if="isRecording" class="recording-indicator">🔴 录音中</div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

const isRecording = ref(false)
const transcript = ref('')
const historyTexts = ref([])

const totalChars = computed(() => historyTexts.value.reduce((sum, t) => sum + t.length, 0))
const totalMinutes = computed(() => Math.max(1, Math.round(totalChars.value / 200)))
const avgSpeed = computed(() => totalMinutes.value > 0 ? Math.round(totalChars.value / totalMinutes.value) : 0)

  let unlisten = null

onMounted(async () => {
  try {
    unlisten = await listen<boolean>('recording-state', async (event) => {
      isRecording.value = event.payload
      if (!event.payload) {
        // Released — trigger transcribe + inject
        await handleTranscribe()
      }
    })
  } catch {
    // browser fallback
  }
})

onUnmounted(() => {
  unlisten?.()
})

async function handleTranscribe() {
  try {
    const settings = await invoke('get_settings')
    // In real implementation, audio_b64 comes from Rust recording buffer
    // For now, placeholder
    const text = await invoke('transcribe_audio', {
      audioB64: '',
      settings,
    })
    transcript.value = text
    await invoke('insert_history', {
      text,
      protocol: settings.protocol || 'openai',
      target_app: '当前应用',
    })
    await invoke('inject_text', { text })
  } catch (e) {
    console.warn('transcribe failed:', e)
  }
}
</script>

<style scoped>
.transcript-text {
  font-size: 14px;
  color: #1A1A1A;
  line-height: 1.6;
  min-height: 60px;
}

.recording-indicator {
  font-size: 13px;
  color: #DC2626;
  font-weight: bold;
  animation: blink 1s ease-in-out infinite;
}

@keyframes blink {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}
</style>
