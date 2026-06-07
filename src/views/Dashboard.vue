<template>
  <div class="dashboard">
    <!-- Header -->
    <div class="header">
      <h1 class="header-title">自然说话，完美书写 – 在任何应用中</h1>
      <div class="badge">
        <span>核心操作：按住 [Right Alt] 键开始录音，松开停止并转换</span>
      </div>
    </div>

    <!-- Stats Grid -->
    <div class="stats-grid">
      <div class="stats-row">
        <div class="stat-card">
          <div class="chart-placeholder"></div>
          <p class="card-title">整体个性化 0%</p>
          <a href="#" class="card-link">查看报告</a>
        </div>
        <div class="stat-card">
          <p class="card-value">1 min</p>
          <p class="card-label">总口述时间</p>
        </div>
      </div>
      <div class="stats-row">
        <div class="stat-card">
          <p class="card-value">117 字</p>
          <p class="card-label">口述字数</p>
        </div>
        <div class="stat-card">
          <p class="card-value">133 字/分钟</p>
          <p class="card-label">平均口述速度</p>
        </div>
      </div>
    </div>

    <!-- Transcript Box -->
    <div class="transcript-box">
      <label class="transcript-label">最近一次转录结果</label>
      <p class="transcript-placeholder">等待语音输入...</p>
    </div>
  </div>
</template>

<script setup>
</script>

<style scoped>
.dashboard {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.header {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.header-title {
  font-family: 'Newsreader', serif;
  font-size: 28px;
  font-weight: bold;
  color: #1A1A1A;
  line-height: 1.3;
}

.badge {
  display: inline-flex;
  align-items: center;
  background: #C8B496;
  color: #FFFFFF;
  padding: 8px 16px;
  border-radius: 20px;
  font-size: 13px;
  width: fit-content;
}

.stats-grid {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.stats-row {
  display: flex;
  gap: 16px;
}

.stat-card {
  flex: 1;
  background: #FFFFFF;
  border: 1px solid #E5E2DD;
  border-radius: 12px;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.chart-placeholder {
  width: 80px;
  height: 80px;
  background: #E0E7FF;
  border-radius: 50%;
}

.card-title {
  font-size: 14px;
  color: #1A1A1A;
}

.card-link {
  font-size: 13px;
  color: #C8B496;
  text-decoration: none;
}

.card-link:hover {
  text-decoration: underline;
}

.card-value {
  font-size: 32px;
  font-weight: bold;
  color: #1A1A1A;
}

.card-label {
  font-size: 14px;
  color: #777777;
}

.transcript-box {
  background: #FFFFFF;
  border: 1px solid #E5E2DD;
  border-radius: 12px;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.transcript-label {
  font-size: 14px;
  font-weight: bold;
  color: #1A1A1A;
}

.transcript-placeholder {
  font-size: 14px;
  color: #777777;
}
</style>
