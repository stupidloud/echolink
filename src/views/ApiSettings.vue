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
          <input type="text" class="form-input" placeholder="https://..." />
        </div>
      </div>

      <div class="form-group">
        <label class="form-label">鉴权密钥 (API Key)</label>
        <div class="input-wrapper">
          <input type="password" class="form-input" value="••••••••" />
          <Eye class="eye-icon" />
        </div>
      </div>

      <div class="form-group">
        <label class="form-label">模型名称 (Model)</label>
        <div class="input-wrapper select-wrapper">
          <span class="select-value">whisper-1</span>
          <ChevronDown class="select-icon" />
        </div>
      </div>
    </div>

    <!-- Save Button -->
    <div class="actions">
      <button class="save-btn">测试连接并保存</button>
    </div>
  </div>
</template>

<script setup>
import { Eye, ChevronDown } from 'lucide-vue-next'
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
  padding: 10px;
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

.select-value {
  font-size: 14px;
  color: #1A1A1A;
}

.select-icon {
  width: 16px;
  height: 16px;
  color: #777777;
}

.actions {
  display: flex;
  justify-content: flex-end;
}

.save-btn {
  background: #C8B496;
  color: #FFFFFF;
  border: none;
  padding: 10px;
  border-radius: 8px;
  font-size: 14px;
  font-weight: bold;
  cursor: pointer;
}
</style>
