<template>
  <div class="history">
    <!-- Toolbar -->
    <div class="toolbar">
      <div class="search-box">
        <Search class="search-icon" />
        <input
          v-model="searchQuery"
          type="text"
          class="search-input"
          placeholder="搜索..."
        />
      </div>
      <div class="time-filter">
        <span>全部</span>
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

      <div v-for="row in filteredRows" :key="row.id" class="table-row">
        <span class="col col-time">{{ row.timestamp }}</span>
        <span class="col col-text">{{ row.text }}</span>
        <span class="col col-protocol">{{ row.protocol }}</span>
        <span class="col col-app">{{ row.target_app }}</span>
        <div class="col col-actions">
          <Copy class="action-icon" @click="copyText(row.text)" />
          <Trash2 class="action-icon" @click="remove(row.id)" />
        </div>
      </div>

      <div v-if="rows.length === 0" class="table-empty">暂无历史记录</div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { Search, ChevronDown, Copy, Trash2 } from 'lucide-vue-next'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'

const rows = ref([])
const searchQuery = ref('')

onMounted(async () => {
  try {
    rows.value = await invoke('get_history', { limit: 50 })
  } catch {
    // browser fallback: empty
  }
})

const filteredRows = computed(() => {
  if (!searchQuery.value) return rows.value
  const q = searchQuery.value.toLowerCase()
  return rows.value.filter(r =>
    r.text.toLowerCase().includes(q) ||
    r.protocol.toLowerCase().includes(q) ||
    r.target_app.toLowerCase().includes(q)
  )
})

async function copyText(text) {
  try {
    await writeText(text)
  } catch {
    // browser fallback: navigator.clipboard
    await navigator.clipboard.writeText(text)
  }
}

async function remove(id) {
  try {
    await invoke('delete_history', { id })
    rows.value = rows.value.filter(r => r.id !== id)
  } catch {
    rows.value = rows.value.filter(r => r.id !== id)
  }
}
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
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
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

.table-empty {
  padding: 40px 16px;
  text-align: center;
  color: #777777;
  font-size: 14px;
}
</style>
