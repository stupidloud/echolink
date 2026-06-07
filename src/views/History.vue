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
  gap: 16px;
  height: 100%;
}

.toolbar {
  display: flex;
  align-items: center;
  gap: 12px;
}

.search-box {
  display: flex;
  align-items: center;
  gap: 8px;
  background: white;
  border: 1px solid #d1d5db;
  border-radius: 6px;
  padding: 8px 12px;
  flex: 1;
}

.search-icon {
  width: 16px;
  height: 16px;
  color: #888;
}

.search-input {
  border: none;
  outline: none;
  font-size: 14px;
  width: 100%;
  color: #1A1A1A;
  background: transparent;
}

.time-filter {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 13px;
  color: #555;
  cursor: pointer;
}

.filter-icon {
  width: 14px;
  height: 14px;
}

.table-container {
  flex: 1;
  overflow-y: auto;
}

.table-header,
.table-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 16px;
  font-size: 13px;
}

.table-header {
  background: white;
  border-radius: 8px 8px 0 0;
  font-weight: 600;
  color: #555;
  border-bottom: 1px solid #e5e7eb;
}

.table-row {
  background: white;
  border-bottom: 1px solid #f3f4f6;
  transition: background 0.15s;
}

.table-row:hover {
  background: #faf9f7;
}

.col { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.col-time { width: 140px; flex-shrink: 0; }
.col-text { flex: 1; }
.col-protocol { width: 100px; flex-shrink: 0; }
.col-app { width: 100px; flex-shrink: 0; }
.col-actions { width: 60px; flex-shrink: 0; display: flex; gap: 8px; }

.action-icon {
  width: 16px;
  height: 16px;
  color: #888;
  cursor: pointer;
  transition: color 0.2s;
}

.action-icon:hover {
  color: #1A1A1A;
}

.table-empty {
  padding: 40px 16px;
  text-align: center;
  color: #888;
  font-size: 14px;
}
</style>
