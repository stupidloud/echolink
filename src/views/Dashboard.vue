<template>
  <div class="dashboard">
    <div class="header">
      <h1 class="header-title">自然说话，完美书写 – 在任何应用中</h1>
      <div class="badge">
        <span>核心操作：按住 [Right Alt] 键开始录音，松开停止并转换</span>
      </div>
    </div>

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

    <div class="transcript-box">
      <label class="transcript-label">最近一次转录结果</label>
      <p class="transcript-text">{{ transcript || '等待语音输入...' }}</p>
      <div v-if="isRecording" class="recording-indicator">
        <span class="rec-dot"></span>录音中 {{ recordingDuration.toFixed(1) }}s
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, emit } from '@tauri-apps/api/event'

const isRecording = ref(false)
const transcript = ref('')
const historyTexts = ref([])
const recordingDuration = ref(0)
const recordingTimer = ref(null)
let mediaRecorder = null
let audioChunks = []
let currentStream = null
const isTranscribing = ref(false)

let audioContext = null
let scriptProcessor = null
let sourceNode = null
let gainNode = null
let analyserNode = null
let levelRafId = null
const pcmChunks = []

const unlistens = []

const totalChars = computed(() => historyTexts.value.reduce((sum, t) => sum + t.length, 0))
const totalMinutes = computed(() => Math.max(1, Math.round(totalChars.value / 200)))
const avgSpeed = computed(() => totalMinutes.value > 0 ? Math.round(totalChars.value / totalMinutes.value) : 0)

onMounted(async () => {
  try {
    historyTexts.value = (await invoke('get_history', { limit: 99999 })).map(r => r.text)
  } catch {
    console.warn('[dashboard] get_history failed')
  }

  try {
    unlistens.push(await listen('recording-state', async (event) => {
      console.log('[event] recording-state →', event.payload)
      if (event.payload === isRecording.value) return
      isRecording.value = event.payload
      if (event.payload) {
        await startRecording()
      } else {
        await stopRecording()
      }
    }))
  } catch {
    console.warn('[dashboard] listen recording-state failed')
  }

  // Frontend key handler: catches AltGr when Echolink has focus
  const onKey = async (e) => {
    if (e.code === 'AltRight' || e.key === 'AltGraph') {
      e.preventDefault()
      if (e.type === 'keydown' && !isRecording.value) {
        console.log('[key] AltGr down (focused)')
        isRecording.value = true
        emit('recording-state', true)
        await startRecording()
      } else if (e.type === 'keyup' && isRecording.value) {
        console.log('[key] AltGr up (focused)')
        isRecording.value = false
        emit('recording-state', false)
        await stopRecording()
      }
    }
  }
  window.addEventListener('keydown', onKey)
  window.addEventListener('keyup', onKey)
  unlistens.push(() => {
    window.removeEventListener('keydown', onKey)
    window.removeEventListener('keyup', onKey)
  })

  try {
    unlistens.push(await listen('transcript-delta', (e) => {
      if (isTranscribing.value) {
        console.log('[event] transcript-delta → +' + e.payload.length + ' chars')
        transcript.value += e.payload
      }
    }))
  } catch {
    console.warn('[dashboard] listen transcript-delta failed')
  }

  try {
    unlistens.push(await listen('transcript-done', async (e) => {
      if (!isTranscribing.value) return
      const text = e.payload
      console.log('[event] transcript-done → final length:', text.length)
      transcript.value = text
      historyTexts.value.push(text)
      try {
        await invoke('insert_history', { text, protocol: settings.protocol || 'openai', targetApp: '当前应用' })
      } catch (e) {
        console.warn('[dashboard] insert_history failed:', e)
      }
      try {
        await invoke('inject_text', { text })
      } catch (e) {
        console.warn('[dashboard] inject_text failed:', e)
      }
      isTranscribing.value = false
    }))
  } catch {
    console.warn('[dashboard] listen transcript-done failed')
  }

  // Pre-request mic permission so it doesn't pop mid-recording
  try {
    const testStream = await navigator.mediaDevices.getUserMedia({
      audio: { channelCount: 1, sampleRate: 16000, echoCancellation: true, noiseSuppression: true }
    })
    testStream.getTracks().forEach(t => t.stop())
    console.log('[mic] permission pre-granted')
  } catch (e) {
    console.warn('[mic] permission denied:', e)
  }
})

onUnmounted(() => {
  for (const un of unlistens) { un() }
})

function startLevelMonitor(ctx, source) {
  analyserNode = ctx.createAnalyser()
  analyserNode.fftSize = 256
  source.connect(analyserNode)
  const dataArray = new Uint8Array(analyserNode.frequencyBinCount)
  let frameCount = 0
  function tick() {
    analyserNode.getByteFrequencyData(dataArray)
    let sum = 0
    for (let i = 0; i < dataArray.length; i++) sum += dataArray[i]
    const avg = sum / dataArray.length / 255
    if (frameCount % 50 === 0) console.log('[audio-level]', avg.toFixed(3))
    frameCount++
    emit('audio-level', avg)
    levelRafId = requestAnimationFrame(tick)
  }
  tick()
}

function stopLevelMonitor() {
  if (levelRafId != null) {
    cancelAnimationFrame(levelRafId)
    levelRafId = null
  }
  if (analyserNode) {
    try { analyserNode.disconnect() } catch {}
    analyserNode = null
  }
}

async function startRecording() {
  try {
    const settings = await invoke('get_settings')
    console.log('[mic] startRecording → protocol:', settings.protocol)
    currentStream = await navigator.mediaDevices.getUserMedia({
      audio: { channelCount: 1, sampleRate: 16000, echoCancellation: true, noiseSuppression: true }
    })
    console.log('[mic] getUserMedia OK, tracks:', currentStream.getAudioTracks().length)
    const protocol = settings.protocol || 'openai'
    if (protocol === 'stepfun') {
      await startPcmRecording(currentStream)
    } else {
      await startWebmRecording(currentStream)
    }
  } catch (e) {
    console.error('[mic] getUserMedia FAILED:', e)
    transcript.value = '⚠️ 无法访问麦克风，请检查权限设置'
  }
}

async function startWebmRecording(stream) {
  audioChunks = []
  mediaRecorder = new MediaRecorder(stream, { mimeType: 'audio/webm' })
  mediaRecorder.ondataavailable = (e) => { if (e.data.size > 0) audioChunks.push(e.data) }
  mediaRecorder.onstop = async () => {
    stopLevelMonitor()
    stream.getTracks().forEach(t => t.stop())
    await handleTranscribeWebM()
  }
  mediaRecorder.start(200)
  const ctx = new AudioContext({ sampleRate: 16000 })
  const src = ctx.createMediaStreamSource(stream)
  startLevelMonitor(ctx, src)
  recordingDuration.value = 0
  recordingTimer.value = setInterval(() => { recordingDuration.value += 0.1 }, 100)
}

async function startPcmRecording(stream) {
  pcmChunks.length = 0
  audioContext = new AudioContext({ sampleRate: 16000 })
  sourceNode = audioContext.createMediaStreamSource(stream)
  scriptProcessor = audioContext.createScriptProcessor(4096, 1, 1)
  gainNode = audioContext.createGain()
  gainNode.gain.value = 0

  scriptProcessor.onaudioprocess = (e) => {
    const input = e.inputBuffer.getChannelData(0)
    const pcm = floatTo16BitPCM(input)
    pcmChunks.push(pcm)
  }
  sourceNode.connect(scriptProcessor)
  scriptProcessor.connect(gainNode)
  gainNode.connect(audioContext.destination)
  startLevelMonitor(audioContext, sourceNode)
  recordingDuration.value = 0
  recordingTimer.value = setInterval(() => { recordingDuration.value += 0.1 }, 100)
}

function floatTo16BitPCM(float32Array) {
  const buffer = new ArrayBuffer(float32Array.length * 2)
  const view = new DataView(buffer)
  for (let i = 0; i < float32Array.length; i++) {
    let s = Math.max(-1, Math.min(1, float32Array[i]))
    view.setInt16(i * 2, s < 0 ? s * 0x8000 : s * 0x7FFF, true)
  }
  return new Uint8Array(buffer)
}

function mergePcmChunks() {
  const totalLen = pcmChunks.reduce((sum, c) => sum + c.length, 0)
  const merged = new Uint8Array(totalLen)
  let offset = 0
  for (const chunk of pcmChunks) {
    merged.set(chunk, offset)
    offset += chunk.length
  }
  return merged
}

async function stopRecording() {
  if (recordingTimer.value) {
    clearInterval(recordingTimer.value)
    recordingTimer.value = null
  }
  stopLevelMonitor()
  const settings = await invoke('get_settings')
  const protocol = settings.protocol || 'openai'
  console.log('[mic] stopRecording → protocol:', protocol)

  if (protocol === 'stepfun') {
    if (gainNode) {
      try { gainNode.disconnect() } catch {}
      gainNode = null
    }
    if (scriptProcessor) {
      try { scriptProcessor.disconnect() } catch {}
      scriptProcessor = null
    }
    if (sourceNode) {
      try { sourceNode.disconnect() } catch {}
      sourceNode = null
    }
    if (audioContext) {
      try { await audioContext.close() } catch {}
      audioContext = null
    }
    if (currentStream) {
      currentStream.getTracks().forEach(t => t.stop())
      currentStream = null
    }
    await handleTranscribePcm()
  } else {
    if (mediaRecorder && mediaRecorder.state !== 'inactive') {
      mediaRecorder.stop()
    }
    if (currentStream) {
      currentStream.getTracks().forEach(t => t.stop())
      currentStream = null
    }
  }
}

async function handleTranscribeWebM() {
  try {
    if (audioChunks.length === 0) return
    const blob = new Blob(audioChunks, { type: 'audio/webm' })
    const base64 = await fileToBase64(blob)
    const settings = await invoke('get_settings')
    console.log('[api] transcribe_audio start → size:', (base64.length * 0.75).toFixed(0), 'bytes, model:', settings.model)
    const text = await invoke('transcribe_audio', { audioB64: base64, settings })
    console.log('[api] transcribe_audio done →', text.length, 'chars')
    transcript.value = text
    await invoke('insert_history', { text, protocol: settings.protocol || 'openai', target_app: '当前应用' })
    await invoke('inject_text', { text })
  } catch (e) {
    console.warn('[api] transcribe_audio failed:', e)
  } finally {
    audioChunks = []
  }
}

async function handleTranscribePcm() {
  try {
    if (pcmChunks.length === 0) return
    const pcmBytes = mergePcmChunks()
    const base64 = arrayBufferToBase64(pcmBytes.buffer)
    const settings = await invoke('get_settings')
    console.log('[api] transcribe_audio_sse start → size:', pcmBytes.length, 'bytes, model:', settings.model)
    isTranscribing.value = true
    transcript.value = ''
    await invoke('transcribe_audio_sse', { audioB64: base64, settings })
  } catch (e) {
    console.warn('[api] transcribe_audio_sse failed:', e)
    isTranscribing.value = false
  } finally {
    pcmChunks.length = 0
  }
}

function arrayBufferToBase64(buffer) {
  const bytes = new Uint8Array(buffer)
  let binary = ''
  const chunkSize = 0x8000
  for (let i = 0; i < bytes.length; i += chunkSize) {
    const chunk = bytes.subarray(i, i + chunkSize)
    binary += String.fromCharCode.apply(null, chunk)
  }
  return btoa(binary)
}

function fileToBase64(blob) {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onloadend = () => {
      const base64 = reader.result.toString().split(',')[1]
      resolve(base64)
    }
    reader.onerror = reject
    reader.readAsDataURL(blob)
  })
}
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
