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
