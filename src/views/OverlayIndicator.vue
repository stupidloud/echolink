<template>
  <div v-show="isRecording && showPill" class="overlay recording">
    <div class="waveform">
      <span v-for="(h, i) in barHeights" :key="i" class="bar" :style="{ height: h + 'px' }"></span>
    </div>
    <div class="status-light"></div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, emit } from '@tauri-apps/api/event'
import { info, warn, error } from '@tauri-apps/plugin-log'
import { currentLocale } from '../i18n'

// This overlay window owns the whole capture + transcription pipeline, so it
// keeps working while the main window is closed/destroyed. It stays shown at all
// times (transparent + click-through), revealing its pill via v-show on
// recording-state. It drives its waveform locally and pushes results to the main
// window (if alive) via events. Logs go through plugin-log so they surface in the
// main window console (prefix [ov]).

const isRecording = ref(false)
const showPill = ref(true)
const barHeights = ref([8, 8, 8, 8, 8])

let mediaRecorder = null
let audioChunks = []
let currentStream = null

let audioContext = null
let scriptProcessor = null
let sourceNode = null
let gainNode = null
let levelProcessor = null
let levelGain = null
const pcmChunks = []

let unlistenRecording = null

onMounted(async () => {
  info('[ov] OverlayIndicator mounted')
  try {
    unlistenRecording = await listen('recording-state', async (event) => {
      info('[ov] recording-state=' + event.payload)
      if (event.payload === isRecording.value) return
      isRecording.value = event.payload
      if (event.payload) {
        // When the main window is focused it shows its own waveform bar, so the
        // desktop pill would be redundant -- hide it (recording still runs).
        try { showPill.value = !(await invoke('is_main_focused')) }
        catch { showPill.value = true }
        barHeights.value = [4, 4, 4, 4, 4]
        await startRecording()
      } else {
        // The pill hides via v-show the moment isRecording flips to false; the
        // window itself stays shown (never suspended) while transcription runs.
        await stopRecording() // capture + transcribe + inject; awaited fully
        barHeights.value = [8, 8, 8, 8, 8]
      }
    })
    info('[ov] recording-state listener attached')
  } catch (e) {
    error('[ov] failed to attach listener: ' + e)
  }
})

onUnmounted(() => {
  if (unlistenRecording) unlistenRecording()
})

// ---- waveform (local, audio-thread driven) ----
function applyLevel(level) {
  if (!isRecording.value) return
  // Noise gate: below the floor the bars stay flat, so silence shows no motion
  // even while Right Alt is held.
  const l = level < 0.04 ? 0 : level
  barHeights.value = Array.from({ length: 5 }, (_, i) => {
    const factor = 1 + Math.sin(i * 1.2) * 0.3
    return Math.max(4, Math.min(24, 4 + l * 32 * factor))
  })
}

function startLevelMonitor(ctx, source) {
  // Level is computed on the audio thread (ScriptProcessor), not requestAnimation
  // Frame, so it keeps flowing regardless of window visibility. gain=0 sink
  // prevents mic feedback.
  levelProcessor = ctx.createScriptProcessor(2048, 1, 1)
  levelGain = ctx.createGain()
  levelGain.gain.value = 0
  let smoothLevel = 0
  levelProcessor.onaudioprocess = (e) => {
    const input = e.inputBuffer.getChannelData(0)
    let sum = 0
    for (let i = 0; i < input.length; i++) sum += input[i] * input[i]
    const rms = Math.sqrt(sum / input.length)
    smoothLevel = smoothLevel * 0.8 + rms * 0.2
    const level = Math.min(1, smoothLevel * 4)
    applyLevel(level)            // local bars
    emit('audio-level', level)   // main window floating-status (when visible)
  }
  source.connect(levelProcessor)
  levelProcessor.connect(levelGain)
  levelGain.connect(ctx.destination)
}

function stopLevelMonitor() {
  if (levelProcessor) {
    levelProcessor.onaudioprocess = null
    try { levelProcessor.disconnect() } catch {}
    levelProcessor = null
  }
  if (levelGain) {
    try { levelGain.disconnect() } catch {}
    levelGain = null
  }
}

// ---- recording ----
async function startRecording() {
  try {
    const settings = await invoke('get_settings')
    const protocol = settings.protocol || 'openai'
    info('[ov] startRecording protocol=' + protocol)
    currentStream = await navigator.mediaDevices.getUserMedia({
      audio: { channelCount: 1, sampleRate: 16000, echoCancellation: true, noiseSuppression: true }
    })
    info('[ov] getUserMedia OK, tracks=' + currentStream.getAudioTracks().length)
    if (protocol === 'stepfun') {
      await startPcmRecording(currentStream)
    } else {
      await startWebmRecording(currentStream)
    }
    info('[ov] capture started')
  } catch (e) {
    error('[ov] startRecording FAILED: ' + e)
    // The tiny overlay can't reliably show a permission prompt; let the main
    // window surface the problem instead.
    try { await emit('mic-denied') } catch {}
  }
}

async function startWebmRecording(stream) {
  audioChunks = []
  mediaRecorder = new MediaRecorder(stream, { mimeType: 'audio/webm' })
  mediaRecorder.ondataavailable = (e) => { if (e.data.size > 0) audioChunks.push(e.data) }
  mediaRecorder.start(200)
  audioContext = new AudioContext({ sampleRate: 16000 })
  try { await audioContext.resume() } catch {}
  sourceNode = audioContext.createMediaStreamSource(stream)
  startLevelMonitor(audioContext, sourceNode)
}

async function startPcmRecording(stream) {
  pcmChunks.length = 0
  audioContext = new AudioContext({ sampleRate: 16000 })
  try { await audioContext.resume() } catch {}
  sourceNode = audioContext.createMediaStreamSource(stream)
  scriptProcessor = audioContext.createScriptProcessor(4096, 1, 1)
  gainNode = audioContext.createGain()
  gainNode.gain.value = 0
  scriptProcessor.onaudioprocess = (e) => {
    const input = e.inputBuffer.getChannelData(0)
    pcmChunks.push(floatTo16BitPCM(input))
  }
  sourceNode.connect(scriptProcessor)
  scriptProcessor.connect(gainNode)
  gainNode.connect(audioContext.destination)
  startLevelMonitor(audioContext, sourceNode)
}

function stopMediaRecorder() {
  return new Promise((resolve) => {
    if (!mediaRecorder || mediaRecorder.state === 'inactive') { resolve(); return }
    mediaRecorder.onstop = () => resolve()
    mediaRecorder.stop()
  })
}

async function stopRecording() {
  stopLevelMonitor()
  const settings = await invoke('get_settings')
  const protocol = settings.protocol || 'openai'
  info('[ov] stopRecording protocol=' + protocol + ' pcmChunks=' + pcmChunks.length + ' webmChunks=' + audioChunks.length)

  if (protocol === 'stepfun') {
    if (scriptProcessor) { try { scriptProcessor.disconnect() } catch {} scriptProcessor = null }
    if (gainNode) { try { gainNode.disconnect() } catch {} gainNode = null }
    if (sourceNode) { try { sourceNode.disconnect() } catch {} sourceNode = null }
    if (audioContext) { try { await audioContext.close() } catch {} audioContext = null }
    if (currentStream) { currentStream.getTracks().forEach(t => t.stop()); currentStream = null }
    await handleTranscribePcm(settings)
  } else {
    await stopMediaRecorder()
    if (audioContext) { try { await audioContext.close() } catch {} audioContext = null }
    if (sourceNode) { try { sourceNode.disconnect() } catch {} sourceNode = null }
    if (currentStream) { currentStream.getTracks().forEach(t => t.stop()); currentStream = null }
    await handleTranscribeWebM(settings)
  }
}

// ---- transcription ----
async function finishTranscript(text, protocol) {
  if (!text) { warn('[ov] finishTranscript: empty text, skipping'); return }
  info('[ov] finishTranscript len=' + text.length + ' → insert_history + inject_text')
  try { await invoke('insert_history', { text, protocol, targetApp: '当前应用' }) }
  catch (e) { warn('[ov] insert_history failed: ' + e) }
  try { await invoke('inject_text', { text }) }
  catch (e) { warn('[ov] inject_text failed: ' + e) }
  try { await emit('history-updated') } catch {}
}

async function handleTranscribeWebM(settings) {
  try {
    if (audioChunks.length === 0) { warn('[ov] webm: no audio captured'); return }
    const blob = new Blob(audioChunks, { type: 'audio/webm' })
    const base64 = await fileToBase64(blob)
    const protocol = settings.protocol || 'openai'
    const cmd = protocol === 'openrouter' ? 'transcribe_audio_openrouter' : 'transcribe_audio'
    info('[ov] ' + cmd + ' start, bytes=' + Math.round(base64.length * 0.75))
    const text = await invoke(cmd, { audioB64: base64, settings, language: currentLocale() })
    info('[ov] ' + cmd + ' done, len=' + (text ? text.length : 0))
    await emit('transcript-done', text) // webm has no streaming; push final to main
    await finishTranscript(text, protocol)
  } catch (e) {
    error('[ov] webm transcribe failed: ' + e)
  } finally {
    audioChunks = []
  }
}

async function handleTranscribePcm(settings) {
  try {
    if (pcmChunks.length === 0) { warn('[ov] pcm: no audio captured'); return }
    const pcmBytes = mergePcmChunks()
    const base64 = arrayBufferToBase64(pcmBytes.buffer)
    info('[ov] transcribe_audio_sse start, bytes=' + pcmBytes.length)
    // The SSE backend streams transcript-delta / transcript-done to all windows
    // itself, so the main window gets the live text; we only need the final.
    const text = await invoke('transcribe_audio_sse', { audioB64: base64, settings, language: currentLocale() })
    info('[ov] transcribe_audio_sse done, len=' + (text ? text.length : 0))
    await finishTranscript(text, settings.protocol || 'stepfun')
  } catch (e) {
    error('[ov] sse transcribe failed: ' + e)
  } finally {
    pcmChunks.length = 0
  }
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
