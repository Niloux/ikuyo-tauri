# Ikuyo-App Rust 后端代码审查报告

## 1. 概述

本文档是对 `ikuyo-app` Tauri 项目中 Rust 后端代码的审查报告。审查范围包括代码设计、架构、性能、可读性等方面。总体而言，项目结构清晰，代码功能明确。本报告旨在提供一系列优化建议，以进一步提升代码质量和应用性能。

## 2. 代码设计与架构 (Code Design & Architecture)

### 2.1 依赖注入与服务层抽象

**现状:**

- `commands` 模块中的函数直接实例化 `BangumiService` 和各个 `Repository`。
- 这种方式导致 `commands` 与具体的服务实现和数据库访问逻辑紧密耦合，不利于测试和未来的功能扩展。

**建议:**

1.  **统一服务层:** 创建一个统一的 `AppServices` 结构体，该结构体持有所有服务的实例（如 `BangumiService`, `CrawlerService` 等）。
2.  **Tauri 状态管理:** 将 `AppServices` 实例作为 Tauri 的托管状态注入。
3.  **在 `commands` 中使用服务:** `commands` 函数通过 `State<AppServices>` 来访问所需的服务，而不是直接创建它们。

**示例:**

```rust
// main.rs or lib.rs
struct AppServices {
    bangumi: BangumiService,
    crawler: CrawlerService,
    // ... other services
}

fn main() {
    // ...
    let app_services = AppServices {
        bangumi: BangumiService::new(/* ... */),
        crawler: CrawlerService::new(/* ... */),
    };
    tauri::Builder::default()
        .manage(app_services)
        // ...
}

// commands/bangumi.rs
#[command]
async fn get_calendar(services: State<'_, AppServices>) -> Result<Vec<BangumiWeekday>, String> {
    services.bangumi.get_calendar().await
}
```

### 2.2 错误处理

**现状:**

- 大部分 `command` 函数返回 `Result<T, String>`，错误信息以字符串形式直接返回给前端。
- 这种方式虽然简单，但缺乏结构化的错误类型，使得前端难以根据错误类型做出相应的处理。

**建议:**

1.  **定义统一的错误枚举:** 创建一个 `AppError` 枚举，包含所有可能的错误类型（如 `DatabaseError`, `ApiError`, `NotFound` 等）。
2.  **实现 `serde::Serialize`:** 为 `AppError` 实现 `Serialize`，以便能作为 JSON 返回给前端。
3.  **实现 `From` Trait:** 为 `AppError` 实现 `From<sqlx::Error>` 和 `From<reqwest::Error>`，简化错误转换。
4.  **更新 `command` 签名:** 将 `command` 的返回类型改为 `Result<T, AppError>`。

**示例:**

```rust
// error.rs
#[derive(Debug, thiserror::Error, serde::Serialize)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("API request failed: {0}")]
    Api(String),
    #[error("Resource not found: {0}")]
    NotFound(String),
    // ... other error types
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Database(err.to_string())
    }
}

// commands/bangumi.rs
#[command]
async fn get_subject(id: i64, pool: State<'_, Arc<SqlitePool>>) -> Result<BangumiSubject, AppError> {
    // ...
}
```

### 2.3 模块职责划分

**现状:**

- `repositories` 模块的职责清晰，负责数据库交互。
- `services` 模块目前主要封装了 API 请求和一些业务逻辑，可以进一步强化其作为业务逻辑核心的地位。
- `commands` 模块应更专注于作为 Tauri 与核心业务逻辑之间的桥梁。

**建议:**

- 将更多涉及多个 `Repository` 操作或复杂计算的逻辑从 `commands` 移至 `services`。例如，`get_episode_availability` 和 `get_episode_resources` 中的逻辑可以移到 `BangumiService` 中。

## 3. 性能优化 (Performance Optimization)

### 3.1 数据库交互

**现状:**

- 在 `get_episode_resources` 等函数中，存在 N+1 查询问题：在循环中为每个资源查询其字幕组名称。
- `insert_many_*` 方法使用了 `QueryBuilder`，这是处理批量插入的好方法。

**建议:**

1.  **预加载或 `JOIN` 查询:**
    - **预加载:** 在获取资源列表后，收集所有 `subtitle_group_id`，然后用 `WHERE id IN (...)` 一次性查询所有字幕组信息，并在内存中进行匹配。
    - **`JOIN` 查询:** 修改 `filter` 查询，使用 `LEFT JOIN` 直接关联 `subtitle_group` 表，一次性获取资源及其字幕组名称。

**示例 (JOIN):**

```sql
-- 在 ResourceRepository::filter 中使用的查询
SELECT r.*, sg.name as group_name
FROM resource r
LEFT JOIN subtitle_group sg ON r.subtitle_group_id = sg.id
WHERE r.mikan_id = ?
-- ... 其他条件
```

### 3.2 并发与异步处理

**现状:**

- `CrawlerService` 中使用了 `buffer_unordered` 来并发处理详情页的抓取，这是一个很好的实践。
- `Worker` 使用 `Semaphore` 来限制并发任务数量，同样值得肯定。

**建议:**

1.  **优化数据库连接池:** 确保 `SqlitePool` 的大小配置合理。对于 SQLite，并发写入是一个挑战，但并发读取是有效的。`CrawlerService` 中的写入操作已经通过事务和缓冲区进行了优化，这是正确的方向。
2.  **减少不必要的 `await`:** 在 `bangumi_cache_refresh_loop` 中，可以考虑将独立的刷新任务（如订阅和非订阅）并行执行。

**示例:**

```rust
// worker.rs
async fn bangumi_cache_refresh_loop(pool: Arc<SqlitePool>, config: Config) {
    // ...
    loop {
        // ...
        let sub_refresh = refresh_all_subscribed_bangumi(&pool, &config);
        let nonsub_refresh = refresh_all_non_subscribed_bangumi(&pool, &config);
        let calendar_refresh = async {
            let service = BangumiService::new(pool.clone(), config.clone());
            let _ = service.get_calendar().await;
        };

        tokio::join!(sub_refresh, nonsub_refresh, calendar_refresh);

        sleep(Duration::from_secs(60)).await;
    }
}
```

## 4. 代码风格与可读性 (Code Style & Readability)

### 4.1 命名与一致性

**现状:**

- 代码整体遵循 Rust 的命名规范。
- `models.rs` 中同时存在 `UserSubscription` (后端模型) 和 `types::subscription::UserSubscription` (前端模型) 的转换，这很好，但可以考虑更清晰的命名来区分，例如 `UserSubscriptionModel` 和 `UserSubscriptionDto`。

### 4.2 配置管理

**现状:**

- `Config` 结构体通过 `load` 方法从 `config.toml` 加载配置，并提供了 `default` 实现，这是很好的实践。
- `BangumiService` 中硬编码了代理地址 `http://127.0.0.1:7890`。

**建议:**

- 将代理地址、Mikanani 基础 URL 等可变配置移入 `config.toml` 文件，使其更易于配置和管理。

### 4.3 日志

**现状:**

- 项目集成了 `tracing` 和 `tracing_appender`，实现了日志的控制台输出和文件记录，非常出色。
- 日志信息较为丰富，有助于调试。

**建议:**

- 在关键的错误路径上增加更详细的日志，例如，在 `command` 的 `Err` 分支中记录详细的错误信息，而不仅仅是返回给前端的简化信息。

## 5. 总结

`ikuyo-app` 的 Rust 后端代码基础良好，展现了对 Rust 和 Tauri 生态的熟练运用。主要的优化空间在于通过依赖注入和服务层抽象来改善代码架构，以及通过 `JOIN` 查询等方式解决 N+1 数据库查询问题来提升性能。

实施以上建议将有助于提高项目的可维护性、可测试性和整体性能。
