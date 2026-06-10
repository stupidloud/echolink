<template>
  <!-- Overlay window (index.html#/overlay) renders ONLY the indicator, no app chrome -->
  <router-view v-if="isOverlay" />
  <div v-else class="app-container">
    <!-- Sidebar -->
    <aside class="sidebar">
      <div class="sidebar-top">
        <svg class="app-icon" viewBox="0 0 40 40" aria-hidden="true">
          <g fill="#C8B496">
            <rect x="4"  y="15" width="4" height="10" rx="2"/>
            <rect x="11" y="10" width="4" height="20" rx="2"/>
            <rect x="18" y="5"  width="4" height="30" rx="2"/>
            <rect x="25" y="10" width="4" height="20" rx="2"/>
            <rect x="32" y="15" width="4" height="10" rx="2"/>
          </g>
        </svg>
        <span class="app-title">Echolink</span>
      </div>

      <nav class="sidebar-menu">
        <router-link to="/dashboard" class="menu-item" active-class="active">
          <Home class="menu-icon" />
          <span>{{ $t('nav.dashboard') }}</span>
        </router-link>
        <router-link to="/history" class="menu-item" active-class="active">
          <History class="menu-icon" />
          <span>{{ $t('nav.history') }}</span>
        </router-link>
        <router-link to="/api-settings" class="menu-item" active-class="active">
          <Server class="menu-icon" />
          <span>{{ $t('nav.apiSettings') }}</span>
        </router-link>
      </nav>

      <div class="sidebar-bottom">
        <div class="lang-switch" role="group" :aria-label="$t('sidebar.language')">
          <button
            v-for="l in ['zh', 'en']"
            :key="l"
            class="lang-btn"
            :class="{ active: locale === l }"
            @click="changeLang(l)"
          >{{ l === 'zh' ? '中' : 'EN' }}</button>
        </div>
        <div class="icon-row">
          <div class="user-avatar"></div>
          <Settings class="icon-btn" />
          <HelpCircle class="icon-btn" />
        </div>
        <div class="pro-card">
          <p class="pro-text">{{ $t('sidebar.proText') }}</p>
          <button class="upgrade-btn">{{ $t('sidebar.learnMore') }}</button>
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
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { Home, History, Server, Settings, HelpCircle } from 'lucide-vue-next'
import { invoke } from '@tauri-apps/api/core'
import { listen, emit } from '@tauri-apps/api/event'
import { setLocale } from './i18n'

// True only inside the standalone overlay window, which loads index.html#/overlay.
// In that window we skip the sidebar/main/floating chrome entirely.
const isOverlay = window.location.hash.startsWith('#/overlay')

const { t, locale } = useI18n()
const changeLang = (l) => setLocale(l)

// The frontend owns all UI text, including the native tray menu: push the
// translated labels to Rust on startup and whenever the language changes. Only the
// main window does this (the overlay shares the same App root but has no chrome).
async function syncTrayLabels() {
  if (isOverlay) return
  try {
    await invoke('set_tray_labels', {
      show: t('tray.show'),
      hide: t('tray.hide'),
      quit: t('tray.quit'),
    })
  } catch {
    // browser fallback / command unavailable
  }
}
watch(locale, syncTrayLabels)

const isRecording = ref(false)
const barHeights = ref([8, 8, 8, 8, 8])

const statusText = computed(() => isRecording.value ? t('status.recording') : t('status.idle'))

  let unlisten = null
  let unlistenLevel = null
  let unlistenKeys = null

  onMounted(async () => {
    syncTrayLabels()
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
        const heights = [4, 4, 4, 4, 4]
        for (let i = 0; i < 5; i++) {
          const factor = 1 + Math.sin(i * 1.2) * 0.3
          heights[i] = Math.max(4, Math.min(24, 4 + level * 32 * factor))
        }
        barHeights.value = heights
      })
    } catch {
      // browser fallback
    }

    // Global key handler: catches AltGr on all pages when focused
    const onKey = async (e) => {
      if (e.code === 'AltRight' || e.key === 'AltGraph') {
        e.preventDefault()
        if (e.type === 'keydown' && !isRecording.value) {
          isRecording.value = true
          emit('recording-state', true)
        } else if (e.type === 'keyup' && isRecording.value) {
          isRecording.value = false
          emit('recording-state', false)
        }
      }
    }
    window.addEventListener('keydown', onKey)
    window.addEventListener('keyup', onKey)
    unlistenKeys = () => {
      window.removeEventListener('keydown', onKey)
      window.removeEventListener('keyup', onKey)
    }
  })

  onUnmounted(() => {
    unlisten?.()
    unlistenLevel?.()
    unlistenKeys?.()
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
  color: #1A1A1A;
}

.app-container {
  display: flex;
  width: 100vw;
  height: 100vh;
  min-width: 960px;
  min-height: 600px;
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
  flex: none;
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

.lang-switch {
  display: flex;
  gap: 4px;
  padding: 4px;
  background: #2A2A2A;
  border-radius: 999px;
  align-self: flex-start;
}

.lang-btn {
  border: none;
  background: transparent;
  color: #777777;
  font-size: 12px;
  font-weight: bold;
  padding: 4px 12px;
  border-radius: 999px;
  cursor: pointer;
  transition: background 0.2s, color 0.2s;
}

.lang-btn.active {
  background: #C8B496;
  color: #FFFFFF;
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
  transition: height 0.12s ease-out;
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
