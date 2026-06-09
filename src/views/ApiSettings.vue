<template>
  <div class="api-settings">
    <div class="form">
      <div class="form-group">
        <label class="form-label">协议类型 (Protocol)</label>
        <div class="input-wrapper select-wrapper">
          <select v-model="form.protocol" class="form-select">
            <option value="stepfun">StepFun SSE（流式，推荐）</option>
            <option value="openai">OpenAI 兼容（HTTP 单次）</option>
            <option value="openrouter">OpenRouter（JSON base64）</option>
          </select>
          <ChevronDown class="select-icon" />
        </div>
      </div>

      <div class="form-group">
        <label class="form-label">接口基础地址 (Base URL)</label>
        <div class="input-wrapper">
          <input
            v-model="form.baseUrl"
            type="text"
            class="form-input"
            placeholder="https://api.stepfun.com/v1"
          />
          <span class="input-hint">可输入到 /v1 层级，如 https://api.stepfun.com/v1</span>
        </div>
      </div>

      <div class="form-group">
        <label class="form-label">鉴权密钥 (API Key)</label>
        <div class="input-wrapper">
          <input
            v-model="form.apiKey"
            :type="showKey ? 'text' : 'password'"
            class="form-input"
            placeholder="sk-..."
          />
          <Eye
            class="eye-icon"
            @click="showKey = !showKey"
          />
        </div>
      </div>

      <div class="form-group">
        <label class="form-label">模型名称 (Model)</label>
        <div ref="comboRef" class="input-wrapper model-combo">
          <input
            v-model="form.model"
            type="text"
            class="form-input"
            placeholder="输入或选择模型 ID"
            @focus="showModels = true"
          />
          <ChevronDown class="select-icon combo-toggle" @click="showModels = !showModels" />
          <ul v-if="showModels && currentModels.length" class="model-dropdown">
            <li
              v-for="m in currentModels"
              :key="m"
              class="model-option"
              :class="{ active: m === form.model }"
              @mousedown.prevent="selectModel(m)"
            >{{ m }}</li>
          </ul>
        </div>
      </div>
    </div>

    <div class="actions">
      <button class="test-btn" @click="test" :disabled="testing">{{ testing ? '测试中...' : '测试连接' }}</button>
      <button class="save-btn" @click="save" :disabled="testing">保存设置</button>
    </div>

    <p v-if="msg" class="result-msg" :class="msgType">{{ msg }}</p>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { Eye, ChevronDown } from 'lucide-vue-next'

const showKey = ref(false)
const msg = ref('')
const msgType = ref('')
const testing = ref(false)
const showModels = ref(false)
const comboRef = ref(null)

function selectModel(m) {
  form.value.model = m
  showModels.value = false
}

function onDocClick(e) {
  if (comboRef.value && !comboRef.value.contains(e.target)) {
    showModels.value = false
  }
}

const modelOptions = {
  stepfun: ['stepaudio-2.5-asr', 'stepaudio-2-asr-pro'],
  openai: ['gpt-4o-mini-transcribe', 'gpt-4o-transcribe', 'whisper-1'],
  openrouter: [
    'openai/gpt-4o-transcribe',
    'openai/gpt-4o-mini-transcribe',
    'openai/whisper-1',
    'openai/whisper-large-v3-turbo',
    'openai/whisper-large-v3',
    'mistralai/voxtral-mini-transcribe',
    'qwen/qwen3-asr-flash-2026-02-10',
    'google/chirp-3',
  ],
}

const defaultForm = {
  protocol: 'stepfun',
  baseUrl: 'https://api.stepfun.com',
  apiKey: '',
  model: 'stepaudio-2.5-asr',
}

const form = ref({ ...defaultForm })

const currentModels = computed(() => modelOptions[form.value.protocol] || modelOptions.stepfun)

onMounted(async () => {
  try {
    const s = await invoke('get_settings')
    form.value = { ...defaultForm, ...s }
    if (!modelOptions[form.value.protocol]) {
      form.value.protocol = 'stepfun'
    }
    if (!currentModels.value.includes(form.value.model)) {
      form.value.model = currentModels.value[0]
    }
  } catch {
    // browser fallback
  }
  document.addEventListener('click', onDocClick)
})

onUnmounted(() => {
  document.removeEventListener('click', onDocClick)
})

async function test() {
  msg.value = '正在测试连接...'
  msgType.value = ''
  testing.value = true
  console.log('[api] test connection →', form.value.baseUrl, form.value.model)
  try {
    const result = await invoke('verify_connection', { settings: form.value })
    console.log('[api] verify_connection OK →', result)
    msg.value = result
    msgType.value = 'success'
  } catch (e) {
    console.log('[api] verify_connection FAILED →', e)
    msg.value = `❌ ${e}`
    msgType.value = 'error'
  } finally {
    testing.value = false
  }
}

async function save() {
  msg.value = '正在保存...'
  msgType.value = ''
  testing.value = true
  console.log('[api] save settings →', form.value.protocol, form.value.baseUrl)
  try {
    await invoke('save_settings', { settings: form.value })
    console.log('[api] save OK')
    msg.value = '✅ 已保存'
    msgType.value = 'success'
  } catch (e) {
    console.log('[api] save FAILED →', e)
    msg.value = `❌ 保存失败: ${e}`
    msgType.value = 'error'
  } finally {
    testing.value = false
  }
}
</script>

<style scoped>
.api-settings {
  display: flex;
  flex-direction: column;
  gap: 24px;
  max-width: 640px;
}

.tabs {
  display: flex;
  gap: 0;
}

.tab {
  padding: 10px 16px;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  cursor: pointer;
  background: #E5E2DD;
  color: #777777;
}

.tab.active {
  background: #C8B496;
  color: #FFFFFF;
}

.form {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.form-label {
  font-size: 14px;
  color: #1A1A1A;
}

.input-wrapper {
  display: flex;
  align-items: center;
  background: #FFFFFF;
  border: 1px solid #E5E2DD;
  border-radius: 8px;
  padding: 10px;
  gap: 8px;
}

.form-input {
  flex: 1;
  border: none;
  outline: none;
  font-size: 14px;
  background: transparent;
  color: #1A1A1A;
}

.form-input::placeholder {
  color: #777777;
}

.eye-icon {
  width: 18px;
  height: 18px;
  color: #777777;
  cursor: pointer;
}

.select-wrapper {
  justify-content: space-between;
  cursor: pointer;
}

.form-select {
  flex: 1;
  border: none;
  outline: none;
  font-size: 14px;
  background: transparent;
  color: #1A1A1A;
  cursor: pointer;
  padding: 0;
  -webkit-appearance: none;
  -moz-appearance: none;
  appearance: none;
}

.select-value {
  font-size: 14px;
  color: #1A1A1A;
}

.select-icon {
  width: 16px;
  height: 16px;
  color: #777777;
}

.combo-toggle {
  cursor: pointer;
}

.model-combo {
  position: relative;
}

.model-dropdown {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  right: 0;
  z-index: 20;
  margin: 0;
  padding: 4px;
  list-style: none;
  background: #FFFFFF;
  border: 1px solid #E5E2DD;
  border-radius: 8px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.12);
  max-height: 220px;
  overflow-y: auto;
}

.model-option {
  padding: 8px 10px;
  font-size: 14px;
  color: #1A1A1A;
  border-radius: 6px;
  cursor: pointer;
}

.model-option:hover {
  background: #F3F4F6;
}

.model-option.active {
  background: #C8B496;
  color: #FFFFFF;
}

.actions {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
}

.test-btn {
  background: white;
  color: #1A1A1A;
  border: 1px solid #E5E2DD;
  padding: 10px 16px;
  border-radius: 8px;
  font-size: 14px;
  cursor: pointer;
}

.test-btn:hover:not(:disabled) {
  background: #f3f4f6;
}

.test-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.save-btn {
  background: #C8B496;
  color: #FFFFFF;
  border: none;
  padding: 10px 16px;
  border-radius: 8px;
  font-size: 14px;
  font-weight: bold;
  cursor: pointer;
}

.save-btn:hover:not(:disabled) {
  opacity: 0.9;
}

.save-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.result-msg {
  font-size: 13px;
}
.result-msg.success {
  color: #16A34A;
}
.result-msg.error {
  color: #DC2626;
}

.input-hint {
  font-size: 11px;
  color: #888;
  margin-top: 2px;
  display: block;
}
</style>
