<template>
  <div class="api-settings">
    <!-- Tabs -->
    <div class="tabs">
      <button class="tab active">OpenAI 听写协议（单次文件投递）</button>
      <button class="tab">流式实时协议（WebSocket 长连接）</button>
    </div>

    <!-- Form -->
    <div class="form">
      <div class="form-group">
        <label class="form-label">接口基础地址 (Base URL)</label>
        <div class="input-wrapper">
          <input
            v-model="form.baseUrl"
            type="text"
            class="form-input"
            placeholder="https://api.openai.com"
          />
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
          <span class="select-value">{{ form.model }}</span>
          <ChevronDown class="select-icon" />
        </div>
      </div>
    </div>

    <!-- Save Button -->
    <div class="actions">
      <button class="save-btn" @click="save">测试连接并保存</button>
    </div>

    <p v-if="msg" class="result-msg" :class="msgType">{{ msg }}</p>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { Eye, ChevronDown } from 'lucide-vue-next'

const showKey = ref(false)
const msg = ref('')
const msgType = ref('')

const defaultForm = {
  baseUrl: 'https://api.stepfun.com',
  apiKey: '',
  model: 'stepaudio-2.5-asr',
  protocol: 'stepfun',
}

const form = ref({ ...defaultForm })

onMounted(async () => {
  try {
    const s = await invoke('get_settings')
    form.value = s
  } catch {
    // browser fallback
  }
})

async function save() {
  msg.value = '正在连接测试...'
  msgType.value = ''
  try {
    await invoke('save_settings', { settings: form.value })
    msg.value = '✅ 已保存'
    msgType.value = 'success'
  } catch (e) {
    msg.value = `❌ 保存失败: ${e}`
    msgType.value = 'error'
  }
}
</script>

<style scoped>
.result-msg {
  font-size: 13px;
  margin-top: 8px;
}
.result-msg.success {
  color: #16A34A;
}
.result-msg.error {
  color: #DC2626;
}
</style>
