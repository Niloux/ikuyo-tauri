# Ikuyo 应用功能集成计划

本文档旨在阐述在 Ikuyo (Tauri+Vue) 应用中集成核心功能的技术方案，包括视频播放器和下载器。

---

## 1. 视频播放器集成方案

### 1.1. 核心技术

- **前端播放器**: **Artplayer.js**
- **文件协议**: Tauri **资源协议 (Asset Protocol)**
- **字幕方案**: 后端使用 **`ffmpeg`** 动态提取内封字幕

### 1.2. 工作流程

1.  **前端请求播放**: 用户点击一个视频文件（尤其是 MKV）。
2.  **Rust 后端处理 (`prepare_video_for_playback` 命令)**:
    a.  接收到视频文件路径。
    b.  **检查文件类型**。如果是 MKV，则在后台调用 `ffmpeg` 命令，从视频中提取出字幕轨道 (`.ass` 或 `.ssa`) 并保存为临时文件。
    c.  返回一个包含**原始视频路径**和**临时字幕路径**的对象给前端。
3.  **前端加载 (Vue)**:
    a.  接收到两个文件路径。
    b.  使用 Tauri 的 `convertFileSrc` API 将这两个路径都转换为 `tauri://` 协议的 URL。
    c.  初始化 Artplayer，将视频 URL 传给 `url` 配置，将字幕 URL 传给 `subtitle.url` 配置。

### 1.3. 优势

- **功能强大**: Artplayer 支持 ASS 特效字幕和弹幕，完美匹配动漫场景。
- **体验无缝**: 用户无需任何额外操作即可看到内封字幕，实现了桌面级播放器的体验。
- **架构清晰**: Rust 负责原生交互 (文件处理、命令执行)，Vue 负责 UI 和播放器控制，分工明确。

---

## 2. 下载器集成方案

为了实现完整的“发现-下载-管理-播放”闭环，应用将集成一个强大的下载器。

### 2.1. 技术选型: `aria2`

我们选择 **`aria2`** 作为后端的下载引擎，并通过 **RPC 接口**进行控制。这是一个极其成熟和强大的方案。

**为何选择 `aria2` 而非原生 BT 库？**

| 对比维度 | 集成 `aria2` (推荐) | 使用原生 BT 库 |
| :--- | :--- | :--- |
| **功能** | **世界级**。支持 BT、磁力、HTTP 等，功能完整。 | **有限**。功能依赖库的实现，通常不完整。 |
| **稳定性** | **极高**。`aria2` 经过长期验证，且**进程隔离**，不影响主应用。 | **风险高**。库的 Bug 可能导致整个应用崩溃。 |
| **开发成本** | **中等**。只需处理进程管理和 RPC 通信，无需关心 BT 协议。 | **极高**。需要自己处理复杂的 BT 协议，费时费力。 |
| **应用体积** | 安装包增大 **2-5 MB** (根据平台)。 | 可能略小，但收益远小于风险。 |

**结论**: 为了功能的完备性、应用的稳定性和开发效率，集成 `aria2` 是压倒性的正确选择。

### 2.2. 架构设计

采用“Rust 后端驱动，`aria2` 进程执行，Vue 前端展示”的模式。

1.  **Rust 后端**: 作为“指挥官”，负责启动、关闭和控制 `aria2` 进程。
2.  **`aria2` 进程**: 作为“士兵”，在后台默默执行所有下载任务。
3.  **Vue 前端**: 作为“仪表盘”，向用户展示任务状态并提供操作入口。

### 2.3. 实施步骤

#### a. 后端实现 (Rust)

1.  **打包 `aria2`**:
    *   下载各平台 (`win-x64`, `darwin-x86_64`, `darwin-aarch64`) 的 `aria2c` 可执行文件。
    *   将它们放入 `src-tauri/bin/` 目录。
    *   在 `tauri.conf.json` 中配置 `bundle.externalBin`，将 `aria2c` 打包进应用。

2.  **管理 `aria2` 进程**:
    *   在 Tauri 应用启动时，由 Rust 代码通过 `tauri::api::process::Command` 启动一个隐藏的 `aria2c` 子进程。
    *   启动命令必须开启 RPC 服务并设置一个安全的 token。
        ```bash
        # 示例命令
        aria2c --enable-rpc --rpc-listen-all=true --rpc-listen-port=6800 --rpc-secret=YOUR_SECRET_TOKEN --rpc-allow-origin-all=true
        ```
    *   在应用退出时，确保能优雅地终止 `aria2c` 进程。

3.  **创建 `aria2` RPC 客户端**:
    *   在 Rust 中创建一个服务模块 (`services/downloader_service.rs`)。
    *   该服务使用 `reqwest` 库向 `http://127.0.0.1:6800/jsonrpc` 发送 POST 请求来调用 `aria2` 的方法。
    *   封装 `aria2.addUri`, `aria2.tellActive`, `aria2.pause`, `aria2.remove` 等核心 RPC 方法。

4.  **暴露 Tauri 命令**:
    *   基于 RPC 客户端，创建一系列清晰的 Tauri 命令供前端调用。
        ```rust
        #[tauri::command]
        async fn add_download(url: String) -> Result<String, String> { /* ... */ }

        #[tauri::command]
        async fn get_all_tasks() -> Result<Vec<DownloadTask>, String> { /* ... */ }
        // ... 其他命令如 pause, resume, remove
        ```

#### b. 前端实现 (Vue)

1.  **状态管理 (`taskStore.ts`)**:
    *   在 Pinia store 中管理 `activeTasks` 和 `completedTasks` 列表。
    *   创建 actions (`fetchTasks`, `startDownload`, `pauseTask` 等) 来调用后端的 Tauri 命令。

2.  **视图 (`TaskManagementView.vue`)**:
    *   使用 `setInterval` 定期 (如每 2 秒) 调用 `fetchTasks` action 来刷新任务列表。
    *   将 store 中的任务数据响应式地渲染到界面上，包括进度条、速度、文件名等。
    *   提供按钮来调用 `pauseTask`, `removeTask` 等操作。
