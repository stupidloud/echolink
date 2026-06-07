# 语音输入法 / Typeless

基于 Editorial Scientific 风格的桌面端语音输入法界面。

## 技术栈

- Vue 3
- Vue Router
- Vite
- Lucide Vue Next (图标库)

## 项目结构

```
vue-project/
├── index.html
├── package.json
├── vite.config.js
└── src/
    ├── main.js
    ├── App.vue
    ├── router.js
    └── views/
        ├── Dashboard.vue      # 首页看板
        ├── ApiSettings.vue    # API 服务器设置
        └── History.vue        # 历史记录
```

## 快速开始

```bash
# 安装依赖
npm install

# 启动开发服务器
npm run dev

# 构建生产版本
npm run build

# 预览生产构建
npm run preview
```

## 页面说明

- `/dashboard` - 首页看板（默认）
- `/api-settings` - API 服务器设置
- `/history` - 历史记录

## 设计风格

- **配色**：羊皮纸金 (Parchment Gold)
- **字体**：Newsreader (标题) + Inter (正文) + IBM Plex Mono (说明)
- **圆角**：基础圆角
- **阴影**：轻柔提升
