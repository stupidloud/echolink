<template>
  <div class="api-settings">
    <div class="form">
      <div class="form-group">
        <label class="form-label">协议类型 (Protocol)</label>
        <div class="input-wrapper select-wrapper">
          <select v-model="form.protocol" class="form-select">
            <option value="stepfun">StepFun SSE（流式，推荐）</option>
            <option value="openai">OpenAI 兼容（HTTP 单次）</option>
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
            placeholder="https://api.stepfun.com"
          />
          <span class="input-hint">请输入到域名，不含具体 API 路径（如 /v1）</span>
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
        <div class="input-wrapper select-wrapper">
          <select v-model="form.model" class="form-select">
            <option v-for="m in modelOptions" :key="m" :value="m">{{ m }}</option>
          </select>
          <ChevronDown class="select-icon" />
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
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { Eye, ChevronDown } from 'lucide-vue-next'

const showKey = ref(false)
const msg = ref('')
const msgType = ref('')
const testing = ref(false)

const modelOptions = {
  stepfun: ['stepaudio-2.5-asr', 'stepaudio-2-asr-pro'],
  openai: ['gpt-4o-mini-transcribe', 'gpt-4o-transcribe', 'whisper-1'],
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
})

async function test() {
  msg.value = '正在测试连接...'
  msgType.value = ''
  testing.value = true
  try {
    const result = await invoke('verify_connection', { settings: form.value })
    msg.value = result
    msgType.value = 'success'
  } catch (e) {
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
  try {
    await invoke('save_settings', { settings: form.value })
    msg.value = '✅ 已保存'
    msgType.value = 'success'
  } catch (e) {
    msg.value = `❌ 保存失败: ${e}`
    msgType.value = 'error'
  } finally {
    testing.value = false
  }
}
</script>

<style scoped>
.input-hint {
  font-size: 11px;
  color: #888;
  margin-top: 4px;
  display: block;
}

.form-select {
  width: 100%;
  padding: 8px 32px 8px 12px;
  font-size: 14px;
  border: 1px solid #d1d5db;
  border-radius: 6px;
  background: white;
  color: #1A1A1A;
  appearance: none;
  cursor: pointer;
}

.actions {
  display: flex;
  gap: 12px;
  margin-top: 20px;
}

.test-btn {
  padding: 10px 20px;
  border: 1px solid #d1d5db;
  border-radius: 6px;
  background: white;
  color: #374151;
  font-size: 14px;
  cursor: pointer;
  transition: background 0.2s;
}

.test-btn:hover:not(:disabled) {
  background: #f3f4f6;
}

.test-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.save-btn {
  padding: 10px 20px;
  border: none;
  border-radius: 6px;
  background: #2563EB;
  color: white;
  font-size: 14px;
  cursor: pointer;
  transition: background 0.2s;
}

.save-btn:hover:not(:disabled) {
  background: #1D4ED8;
}

.save-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.result-msg {
  font-size: 13px;
  margin-top: 12px;
}
.result-msg.success {
  color: #16A34A;
}
.result-msg.error {
  color: #DC2626;
}
</style>
