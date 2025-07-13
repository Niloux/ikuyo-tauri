# Tauri + Vue + TypeScript

This template should help get you started developing with Vue 3 and TypeScript in Vite. The template uses Vue 3 `<script setup>` SFCs, check out the [script setup docs](https://v3.vuejs.org/api/sfc-script-setup.html#sfc-script-setup) to learn more.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Volar](https://marketplace.visualstudio.com/items?itemName=Vue.volar) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## Type Support For `.vue` Imports in TS

Since TypeScript cannot handle type information for `.vue` imports, they are shimmed to be a generic Vue component type by default. In most cases this is fine if you don't really care about component prop types outside of templates. However, if you wish to get actual prop types in `.vue` imports (for example to get props validation when using manual `h(...)` calls), you can enable Volar's Take Over mode by following these steps:

1. Run `Extensions: Show Built-in Extensions` from VS Code's command palette, look for `TypeScript and JavaScript Language Features`, then right click and select `Disable (Workspace)`. By default, Take Over mode will enable itself if the default TypeScript extension is disabled.
2. Reload the VS Code window by running `Developer: Reload Window` from the command palette.

You can learn more about Take Over mode [here](https://github.com/johnsoncodehk/volar/discussions/471).

## 环境区分与初始化说明

- 开发环境：
  - 启动命令：`pnpm tauri dev`
  - 数据库文件：`src-tauri/ikuyo.db`
  - 日志文件：`src-tauri/logs/ikuyo.log`
  - 数据库和日志文件均自动初始化，无需手动操作。

- 生产环境：
  - 启动命令：`pnpm tauri build`（或打包后运行）
  - 数据库文件：应用数据目录下 `ikuyo.db`
  - 日志文件：应用数据目录下 `logs/ikuyo.log`
  - 数据库和日志文件均自动初始化，无需手动操作。

- 表结构变更：
  - 只需修改 `src-tauri/schema.sql`，所有新环境自动按最新表结构初始化。
  - 生产环境如需重建数据库，删除数据库文件后重启应用即可。
- schema.sql 已编译进二进制，生产环境无需关心文件路径或分发问题。

## 未来的开发计划
- 下载器的集成，预计用aria2c。
- 播放器的集成，方案还未确定。
