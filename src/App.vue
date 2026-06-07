<template>
  <div class="app-container">
    <!-- Sidebar -->
    <aside class="sidebar">
      <div class="sidebar-top">
        <div class="app-icon"></div>
        <span class="app-title">Echolink</span>
      </div>

      <nav class="sidebar-menu">
        <router-link to="/dashboard" class="menu-item" active-class="active">
          <Home class="menu-icon" />
          <span>首页看板</span>
        </router-link>
        <router-link to="/history" class="menu-item" active-class="active">
          <History class="menu-icon" />
          <span>历史记录</span>
        </router-link>
        <router-link to="/api-settings" class="menu-item" active-class="active">
          <Server class="menu-icon" />
          <span>API 服务器设置</span>
        </router-link>
      </nav>

      <div class="sidebar-bottom">
        <div class="icon-row">
          <div class="user-avatar"></div>
          <Settings class="icon-btn" />
          <HelpCircle class="icon-btn" />
        </div>
        <div class="pro-card">
          <p class="pro-text">按住 Right Alt 开始录音，松开停止并转换</p>
          <button class="upgrade-btn">了解更多</button>
        </div>
      </div>
    </aside>

    <!-- Main Content -->
    <main class="main-content">
      <router-view />
    </main>

    <!-- Floating Status Bar -->
    <div class="floating-status" :class="{ recording: isRecording }">
      <div class="waveform">
        <span v-for="(h, i) in barHeights" :key="i" class="bar" :style="{ height: h + 'px', animationDelay: `${i * 0.15}s` }"></span>
      </div>
      <div class="status-light"></div>
      <span class="status-text">{{ statusText }}</span>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { Home, History, Server, Settings, HelpCircle } from 'lucide-vue-next'
import { listen } from '@tauri-apps/api/event'

const isRecording = ref(false)
const barHeights = ref([8, 8, 8, 8, 8])

const statusText = computed(() => {
  if (isRecording.value) return '正在录音...'
  return '按住 Right Alt 开始语音输入'
})

  let unlisten = null
  let unlistenLevel = null

  onMounted(async () => {
    try {
      unlisten = await listen('recording-state', (event) => {
        isRecording.value = event.payload
        if (!event.payload) barHeights.value = [8, 8, 8, 8, 8]
      })
    } catch {
      // browser fallback
    }
    try {
      unlistenLevel = await listen('audio-level', (event) => {
        const level = event.payload
        barHeights.value = Array.from({ length: 5 }, () =>
          Math.max(4, Math.min(24, 4 + level * 40 + Math.random() * 8))
        )
      })
    } catch {
      // browser fallback
    }
  })

  onUnmounted(() => {
    unlisten?.()
    unlistenLevel?.()
  })
</script>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  background: #F4F2EF;
  color: #1A1A1A;
}

.app-container {
  display: flex;
  width: 1280px;
  height: 800px;
  margin: 0 auto;
  background: #F4F2EF;
  position: relative;
}

/* Sidebar */
.sidebar {
  width: 240px;
  height: 100%;
  background: #1A1A1A;
  display: flex;
  flex-direction: column;
  padding: 20px;
  gap: 16px;
}

.sidebar-top {
  display: flex;
  align-items: center;
  gap: 12px;
}

.app-icon {
  width: 40px;
  height: 40px;
  background: #3B82F6;
  border-radius: 50%;
}

.app-title {
  font-size: 16px;
  font-weight: bold;
  color: #FFFFFF;
}

.sidebar-menu {
  display: flex;
  flex-direction: column;
  gap: 8px;
  flex: 1;
}

.menu-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px;
  color: #E5E2DD;
  text-decoration: none;
  border-radius: 6px;
  transition: background 0.2s;
}

.menu-item:hover {
  background: #2A2A2A;
}

.menu-item.active {
  background: #2A2A2A;
  color: #FFFFFF;
}

.menu-icon {
  width: 20px;
  height: 20px;
  color: #777777;
}

.menu-item.active .menu-icon {
  color: #C8B496;
}

.sidebar-bottom {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.icon-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.user-avatar {
  width: 32px;
  height: 32px;
  background: #6366F1;
  border-radius: 50%;
}

.icon-btn {
  width: 20px;
  height: 20px;
  color: #777777;
  cursor: pointer;
}

.pro-card {
  background: #2A2A2A;
  padding: 12px;
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.pro-text {
  font-size: 12px;
  color: #E5E2DD;
  line-height: 1.4;
}

.upgrade-btn {
  background: #C8B496;
  color: #FFFFFF;
  border: none;
  padding: 6px 12px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: bold;
  cursor: pointer;
  align-self: flex-start;
}

/* Main Content */
.main-content {
  flex: 1;
  height: 100%;
  overflow-y: auto;
  padding: 24px;
}

/* Floating Status Bar */
.floating-status {
  position: absolute;
  bottom: 24px;
  left: 50%;
  transform: translateX(-50%);
  background: #1A1A1A;
  padding: 12px 20px;
  border-radius: 24px;
  display: flex;
  align-items: center;
  gap: 12px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
  transition: background 0.3s;
}

.floating-status.recording {
  background: #2A1A1A;
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
  transition: height 0.08s ease-out;
}

.recording .bar {
  background: #EF4444;
}

.status-light {
  width: 12px;
  height: 12px;
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

.status-text {
  font-size: 13px;
  color: #E5E2DD;
  white-space: nowrap;
}
</style>
