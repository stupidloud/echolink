<template>
  <div class="app-container">
    <!-- Sidebar -->
    <aside class="sidebar">
      <div class="sidebar-top">
        <div class="app-icon"></div>
        <span class="app-title">语音输入法 / Typeless</span>
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
          <p class="pro-text">Pro版本试用中：已使用 6 天 / 共 30 天</p>
          <button class="upgrade-btn">升级</button>
        </div>
      </div>
    </aside>

    <!-- Main Content -->
    <main class="main-content">
      <router-view />
    </main>

    <!-- Floating Status Bar -->
    <div class="floating-status">
      <div class="waveform"></div>
      <div class="status-light"></div>
      <span class="status-text">实时流式文本上屏预览...</span>
    </div>
  </div>
</template>

<script setup>
import { Home, History, Server, Settings, HelpCircle } from 'lucide-vue-next'
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
  padding: 12px;
  border-radius: 24px;
  display: flex;
  align-items: center;
  gap: 12px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
}

.waveform {
  width: 120px;
  height: 24px;
  background: #2A2A2A;
  border-radius: 4px;
}

.status-light {
  width: 12px;
  height: 12px;
  background: #C8B496;
  border-radius: 50%;
}

.status-text {
  font-size: 13px;
  color: #E5E2DD;
}
</style>
