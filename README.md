<div align="center">
  <img src="src/assets/ikuyo-avatar.png" width="150" alt="Ikuyo Logo">
  <h1>IKUYO</h1>
  <p>一个优雅、集成化的动漫追番与资源管理桌面应用。</p>
  <p>专为希望自动化追番流程、聚合资源并在一个地方完成所有操作的动漫爱好者打造。</p>
</div>

<p align="center">
  <a href="https://github.com/Niloux/ikuyo-tauri/actions/workflows/tauri-build.yml"><img alt="Build Status" src="https://img.shields.io/github/actions/workflow/status/Niloux/ikuyo-tauri/tauri-build.yml?branch=main&style=for-the-badge"></a>
  <a href="https://github.com/Niloux/ikuyo-tauri/releases/latest"><img alt="Latest Release" src="https://img.shields.io/github/v/release/Niloux/ikuyo-tauri?style=for-the-badge&color=blue"></a>
  <a href="https://github.com/Niloux/ikuyo-tauri/releases"><img alt="Total Downloads" src="https://img.shields.io/github/downloads/Niloux/ikuyo-tauri/total?style=for-the-badge&color=green"></a>
  <a href="LICENSE"><img alt="License" src="https://img.shields.io/badge/license-MIT-green.svg?style=for-the-badge"></a>
</p>

---

## 功能亮点

- **每周放送**: 以周视图清晰展示当前季度的番剧更新情况，不错过任何一集。
- **智能订阅与抓取**: 一键订阅你喜欢的番剧，Ikuyo 会自动从资源站 (如 Mikan Project) 抓取最新发布的剧集资源。
- **内置下载管理**: 集成下载功能，从资源抓取到下载一气呵成，并提供完整的下载任务管理。
- **资源库**: 自动整理已下载的剧集文件，形成个人本地资源库，方便随时回顾。
- **便捷播放**: 直接调用系统默认播放器播放已下载的视频，无缝衔接观看体验。
- **更新提醒**: (待开发) 当你订阅的番剧更新时，发送桌面通知提醒你。
- **现代且美观的 UI**: 基于 Vue 3 和精心设计的组件，提供流畅、直观的用户体验。

## 技术栈

Ikuyo 是一个现代化的桌面应用，充分利用了 Web 技术和原生性能的优势。

- **核心框架**: **[Tauri](https://tauri.app/)** - 使用 Web 前端构建轻量、快速、安全的跨平台桌面应用。
- **后端**: **[Rust](https://www.rust-lang.org/)** - 提供内存安全、高性能的后端逻辑，负责核心业务如爬虫、数据库和文件系统操作。
- **前端**:
  - **[Vue 3](https.vuejs.org/)** (Composition API) - 渐进式 JavaScript 框架，用于构建用户界面。
  - **[TypeScript](https://www.typescriptlang.org/)** - 为 JavaScript 添加静态类型，提升代码质量和可维护性。
  - **[Vite](https://vitejs.dev/)** - 下一代前端构建工具，提供极速的开发体验。
  - **[Pinia](https://pinia.vuejs.org/)** - Vue 的官方状态管理库。
- **数据库**: **[SQLite](https://www.sqlite.org/index.html)** - 轻量级的本地文件数据库，用于存储订阅、任务等应用数据。

## 快速开始 (本地开发)

想要参与开发或自行构建？请遵循以下步骤：

1.  **环境准备**:
    - 确保你已经安装了 [Node.js](https://nodejs.org/en/) 和 [pnpm](https://pnpm.io/installation)。
    - 根据 [Tauri 官网](https://tauri.app/) 设置好 Rust 和系统依赖。

2.  **克隆仓库**:

    ```bash
    git clone https://github.com/Niloux/ikuyo-tauri.git
    cd ikuyo-tauri
    ```

3.  **安装依赖**:

    ```bash
    pnpm install
    ```

4.  **启动开发环境**:
    ```bash
    pnpm tauri dev
    ```
    此命令将同时启动 Vite 前端开发服务器和 Tauri 后端。

## 贡献

欢迎任何形式的贡献！无论是提交 Issue、请求新功能还是发送 Pull Request。

## 许可证

本项目基于 [MIT](LICENSE) 许可证。
