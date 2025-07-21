# Ikuyo - 动漫追番、下载与管理工具

Ikuyo 是一款专为动漫爱好者打造的桌面应用，提供一站式的追番、资源搜索、下载和播放体验。

## ✨ 核心功能

- **新番日历**: 按周展示当前季度的新番列表，轻松追踪更新。
- **动漫详情**: 查看动漫的详细信息、剧集列表和封面。
- **资源搜索**: 内置资源爬虫，可从蜜柑计划获取动漫资源。
- **订阅管理**: 订阅你喜欢的动漫，自动追踪更新并可配置自动下载。
- **下载管理**: 集成下载功能，轻松管理下载任务(开发中)。
- **内置播放器**: 无需离开应用，直接播放已下载的视频(开发中)。
- **任务中心**: 查看和管理正在进行的后台任务，如资源抓取和下载。

## 🚀 技术栈

- **前端**:
  - [Vue 3](https://vuejs.org/)
  - [TypeScript](https://www.typescriptlang.org/)
  - [Vite](https://vitejs.dev/)
  - [Pinia](https://pinia.vuejs.org/)
- **核心与桌面端**:
  - [Rust](https://www.rust-lang.org/)
  - [Tauri](https://tauri.app/)
- **数据库**:
  - SQLite

## 🛠️ 如何开始

**环境准备**

在开始之前，请确保你已经安装了 [Node.js](https://nodejs.org/) (推荐使用 pnpm) 和 [Rust](https://www.rust-lang.org/tools/install)。

**安装与启动**

1.  **克隆仓库**
    ```bash
    git clone https://github.com/your-username/ikuyo-app.git
    cd ikuyo-app
    ```

2.  **安装依赖**
    ```bash
    pnpm install
    ```

3.  **启动开发环境**
    ```bash
    pnpm tauri dev
    ```
    此命令将同时启动前端开发服务器和 Tauri 后端。

4.  **打包应用**
    ```bash
    pnpm tauri build
    ```
    打包后的应用位于 `src-tauri/target/release/bundle/` 目录下。

## 📁 项目结构

```
.
├── src/                      # 前端代码 (Vue.js)
│   ├── components/           # 可复用组件
│   ├── views/                # 页面视图
│   ├── stores/               # Pinia 状态管理
│   ├── services/             # API 服务封装
│   ├── router/               # 路由配置
│   └── main.ts               # 应用入口
└── src-tauri/                # 后端代码 (Rust)
    ├── src/
    │   ├── commands/         # Tauri 命令
    │   ├── services/         # 核心业务逻辑
    │   ├── repositories/     # 数据库操作
    │   ├── core/             # 核心模块 (如爬虫、解析器)
    │   └── main.rs           # Rust 应用入口
    ├── migrations/           # 数据库迁移文件
    └── tauri.conf.json       # Tauri 配置
```