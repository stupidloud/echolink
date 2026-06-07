<template>
  <div class="history">
    <!-- Toolbar -->
    <div class="toolbar">
      <div class="search-box">
        <Search class="search-icon" />
        <input type="text" class="search-input" placeholder="搜索..." />
      </div>
      <div class="time-filter">
        <span>今天</span>
        <ChevronDown class="filter-icon" />
      </div>
    </div>

    <!-- Table -->
    <div class="table-container">
      <div class="table-header">
        <span class="col col-time">时间戳</span>
        <span class="col col-text">转录文本片段预览</span>
        <span class="col col-protocol">所用接口协议</span>
        <span class="col col-app">目标灌入应用</span>
        <span class="col col-actions">操作</span>
      </div>

      <div class="table-row">
        <span class="col col-time">2026-06-07 14:32</span>
        <span class="col col-text">今天天气真不错，适合出去散步...</span>
        <span class="col col-protocol">OpenAI 听写协议</span>
        <span class="col col-app">记事本</span>
        <div class="col col-actions">
          <Copy class="action-icon" />
          <Trash2 class="action-icon" />
        </div>
      </div>

      <div class="table-row">
        <span class="col col-time">2026-06-07 10:15</span>
        <span class="col col-text">请帮我安排明天下午的会议...</span>
        <span class="col col-protocol">流式实时协议</span>
        <span class="col col-app">微信</span>
        <div class="col col-actions">
          <Copy class="action-icon" />
          <Trash2 class="action-icon" />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { Search, ChevronDown, Copy, Trash2 } from 'lucide-vue-next'
</script>

<style scoped>
.history {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.toolbar {
  display: flex;
  gap: 12px;
}

.search-box {
  display: flex;
  align-items: center;
  background: #FFFFFF;
  border: 1px solid #E5E2DD;
  border-radius: 8px;
  padding: 10px;
  gap: 8px;
  width: 280px;
}

.search-icon {
  width: 18px;
  height: 18px;
  color: #777777;
}

.search-input {
  flex: 1;
  border: none;
  outline: none;
  font-size: 14px;
  background: transparent;
  color: #1A1A1A;
}

.search-input::placeholder {
  color: #777777;
}

.time-filter {
  display: flex;
  align-items: center;
  background: #FFFFFF;
  border: 1px solid #E5E2DD;
  border-radius: 8px;
  padding: 10px;
  gap: 8px;
  cursor: pointer;
  font-size: 14px;
  color: #1A1A1A;
}

.filter-icon {
  width: 16px;
  height: 16px;
  color: #777777;
}

.table-container {
  background: #FFFFFF;
  border: 1px solid #E5E2DD;
  border-radius: 12px;
  overflow: hidden;
}

.table-header {
  display: flex;
  background: #F4F2EF;
  padding: 12px;
  border-bottom: 1px solid #E5E2DD;
}

.table-row {
  display: flex;
  padding: 12px;
  border-bottom: 1px solid #E5E2DD;
}

.table-row:last-child {
  border-bottom: none;
}

.col {
  font-size: 13px;
  display: flex;
  align-items: center;
}

.col-time {
  width: 140px;
  color: #4A4A4A;
  font-weight: bold;
}

.col-text {
  width: 360px;
  color: #1A1A1A;
}

.col-protocol {
  width: 160px;
  color: #1A1A1A;
}

.col-app {
  width: 160px;
  color: #1A1A1A;
}

.col-actions {
  width: 100px;
  justify-content: flex-end;
  gap: 8px;
}

.action-icon {
  width: 16px;
  height: 16px;
  color: #777777;
  cursor: pointer;
}

.action-icon:hover {
  color: #C8B496;
}
</style>
