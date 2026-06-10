<template>
  <div class="dashboard">
    <div class="header">
      <h1 class="header-title">{{ $t('dashboard.title') }}</h1>
      <div class="badge">
        <span>{{ $t('dashboard.badge') }}</span>
      </div>
    </div>

    <div class="stats-grid">
      <div class="stats-row">
        <div class="stat-card">
          <div class="chart-placeholder"></div>
          <p class="card-title">{{ $t('dashboard.personalization') }}</p>
          <a href="#" class="card-link">{{ $t('dashboard.viewReport') }}</a>
        </div>
        <div class="stat-card">
          <p class="card-value">{{ totalMinutes }} min</p>
          <p class="card-label">{{ $t('dashboard.totalTime') }}</p>
        </div>
      </div>
      <div class="stats-row">
        <div class="stat-card">
          <p class="card-value">{{ totalChars }} {{ $t('dashboard.unitChars') }}</p>
          <p class="card-label">{{ $t('dashboard.dictatedChars') }}</p>
        </div>
        <div class="stat-card">
          <p class="card-value">{{ avgSpeed }} {{ $t('dashboard.unitSpeed') }}</p>
          <p class="card-label">{{ $t('dashboard.avgSpeed') }}</p>
        </div>
      </div>
    </div>

    <div class="transcript-box">
      <label class="transcript-label">{{ $t('dashboard.lastTranscript') }}</label>
      <p class="transcript-text">{{ transcript || $t('dashboard.waiting') }}</p>
      <div v-if="isRecording" class="recording-indicator">
        <span class="rec-dot"></span>{{ $t('dashboard.recording') }} {{ recordingDuration.toFixed(1) }}s
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

const { t } = useI18n()

// Recording + transcription now live in the overlay window (so they survive the
// main window being closed/destroyed). The dashboard is a pure display: it
// reflects recording state and shows transcripts pushed via events, and never
// touches the microphone itself.

const isRecording = ref(false)
const transcript = ref('')
const historyTexts = ref([])
const recordingDuration = ref(0)
let durTimer = null

const unlistens = []

const totalChars = computed(() => historyTexts.value.reduce((sum, t) => sum + t.length, 0))
const totalMinutes = computed(() => Math.max(1, Math.round(totalChars.value / 200)))
const avgSpeed = computed(() => totalMinutes.value > 0 ? Math.round(totalChars.value / totalMinutes.value) : 0)

async function refreshHistory() {
  try {
    historyTexts.value = (await invoke('get_history', { limit: 99999 })).map(r => r.text)
  } catch {
    console.warn('[dashboard] get_history failed')
  }
}

onMounted(async () => {
  await refreshHistory()

  try {
    unlistens.push(await listen('recording-state', (event) => {
      isRecording.value = event.payload
      if (event.payload) {
        transcript.value = ''
        recordingDuration.value = 0
        if (durTimer) clearInterval(durTimer)
        durTimer = setInterval(() => { recordingDuration.value += 0.1 }, 100)
      } else if (durTimer) {
        clearInterval(durTimer)
        durTimer = null
      }
    }))
  } catch {
    console.warn('[dashboard] listen recording-state failed')
  }

  try {
    unlistens.push(await listen('transcript-delta', (e) => { transcript.value += e.payload }))
  } catch {}
  try {
    unlistens.push(await listen('transcript-done', (e) => { transcript.value = e.payload }))
  } catch {}
  try {
    unlistens.push(await listen('history-updated', () => { refreshHistory() }))
  } catch {}
  try {
    unlistens.push(await listen('mic-denied', () => {
      transcript.value = t('dashboard.micDenied')
    }))
  } catch {}

  // Pre-warm mic permission in this large window so the tiny overlay never has
  // to surface a permission prompt; the same-origin grant is shared + persisted.
  try {
    const s = await navigator.mediaDevices.getUserMedia({
      audio: { channelCount: 1, sampleRate: 16000, echoCancellation: true, noiseSuppression: true }
    })
    s.getTracks().forEach(t => t.stop())
    console.log('[mic] permission pre-granted')
  } catch (e) {
    console.warn('[mic] permission denied:', e)
  }
})

onUnmounted(() => {
  if (durTimer) clearInterval(durTimer)
  for (const un of unlistens) { un() }
})
</script>

<style scoped>
.dashboard {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.header {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.header-title {
  font-family: 'Newsreader', serif;
  font-size: 28px;
  font-weight: bold;
  color: #1A1A1A;
  line-height: 1.3;
}

.badge {
  display: inline-flex;
  align-items: center;
  background: #C8B496;
  color: #FFFFFF;
  padding: 8px 16px;
  border-radius: 20px;
  font-size: 13px;
  width: fit-content;
}

.stats-grid {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.stats-row {
  display: flex;
  gap: 16px;
}

.stat-card {
  flex: 1;
  background: #FFFFFF;
  border: 1px solid #E5E2DD;
  border-radius: 12px;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.chart-placeholder {
  width: 80px;
  height: 80px;
  background: #E0E7FF;
  border-radius: 50%;
}

.card-title {
  font-size: 14px;
  color: #1A1A1A;
}

.card-link {
  font-size: 13px;
  color: #C8B496;
  text-decoration: none;
}

.card-link:hover {
  text-decoration: underline;
}

.card-value {
  font-size: 32px;
  font-weight: bold;
  color: #1A1A1A;
}

.card-label {
  font-size: 14px;
  color: #777777;
}

.transcript-box {
  background: #FFFFFF;
  border: 1px solid #E5E2DD;
  border-radius: 12px;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  flex: 1;
}

.transcript-label {
  font-size: 14px;
  font-weight: bold;
  color: #1A1A1A;
}

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
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 4px;
  animation: blink 1s ease-in-out infinite;
}

.rec-dot {
  width: 10px;
  height: 10px;
  background: #DC2626;
  border-radius: 50%;
  animation: pulse 0.8s ease-in-out infinite;
}

@keyframes blink {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}

@keyframes pulse {
  0%, 100% { transform: scale(1); opacity: 1; }
  50% { transform: scale(1.3); opacity: 0.6; }
}
</style>
