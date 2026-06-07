# EchoLink

基于 Editorial Scientific 风格的桌面端语音输入法界面。

## 技术栈

- Vue 3 + Vue Router
- Vite（前端构建）
- Tauri（桌面外壳，Rust 后端）
- Lucide Vue Next（图标库）

## 本地开发（仅前端，无需 Rust）

```bash
# 安装依赖
npm install

# 启动前端开发服务器（端口 3000）
npm run dev

# 构建前端产物（用于预览 / 提交 CI）
npm run build
```

> 注意：本地**不安装** Rust 工具链。`tauri dev` / `tauri build` 等命令由 CI 执行。

## 项目结构

```
echolink/
├── src/                    # Vue 前端
│   ├── App.vue
│   ├── main.js
│   ├── router.js
│   └── views/
│       ├── Dashboard.vue      # 首页看板
│       ├── ApiSettings.vue    # API 服务器设置
│       └── History.vue        # 历史记录
├── src-tauri/              # Tauri / Rust 后端（CI 编译）
│   ├── tauri.conf.json
│   └── src/
├── index.html
├── package.json
└── vite.config.js
```

## 页面说明

- `/dashboard` — 首页看板（默认）
- `/api-settings` — API 服务器设置
- `/history` — 历史记录

## CI 编译

- 平台：Linux (AppImage) / macOS (dmg) / Windows (msi)
- 触发：push 到 `main` 分支 + PR
- 工作流：`.github/workflows/build.yml`
- 产物：Actions Artifacts（暂不自动发布 Release）

## 设计风格

- **配色**：羊皮纸金 (Parchment Gold)
- **字体**：Newsreader (标题) + Inter (正文) + IBM Plex Mono (说明)
- **圆角**：基础圆角
- **阴影**：轻柔提升
