-- Anime表
CREATE TABLE IF NOT EXISTS anime (
    mikan_id INTEGER PRIMARY KEY,
    bangumi_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    original_title TEXT,
    broadcast_day TEXT,
    broadcast_start INTEGER,
    official_website TEXT,
    bangumi_url TEXT,
    description TEXT,
    status TEXT DEFAULT 'unknown', -- Rust端为AnimeStatus枚举，需与TEXT序列化一致
    created_at INTEGER,
    updated_at INTEGER
);

-- Add indexes as defined in SQLModel
CREATE INDEX IF NOT EXISTS idx_anime_bangumi_id ON anime (bangumi_id);
CREATE INDEX IF NOT EXISTS idx_anime_title ON anime (title);

-- CrawlerTask表
CREATE TABLE IF NOT EXISTS crawler_task (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_type TEXT NOT NULL, -- Rust端为CrawlerTaskType枚举
    status TEXT NOT NULL,    -- Rust端为CrawlerTaskStatus枚举
    parameters TEXT,
    result_summary TEXT,
    created_at INTEGER,
    started_at INTEGER,
    completed_at INTEGER,
    error_message TEXT,
    percentage REAL,
    processed_items INTEGER,
    total_items INTEGER,
    processing_speed REAL,
    estimated_remaining REAL
);

CREATE INDEX IF NOT EXISTS idx_crawler_task_created_at ON crawler_task (created_at);
-- CREATE INDEX IF NOT EXISTS idx_crawler_task_worker_pid ON crawler_task (worker_pid); -- 若worker_pid移除则注释

-- AnimeSubtitleGroup表
CREATE TABLE IF NOT EXISTS anime_subtitle_group (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    mikan_id INTEGER NOT NULL,
    subtitle_group_id INTEGER NOT NULL,
    first_release_date INTEGER,
    last_update_date INTEGER,
    resource_count INTEGER DEFAULT 0,
    is_active BOOLEAN DEFAULT true,
    created_at INTEGER,
    updated_at INTEGER,
    FOREIGN KEY (mikan_id) REFERENCES anime (mikan_id),
    FOREIGN KEY (subtitle_group_id) REFERENCES subtitle_group (id)
);

-- ScheduledJob表
CREATE TABLE IF NOT EXISTS scheduled_job (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    job_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    cron_expression TEXT NOT NULL,
    crawler_mode TEXT,
    parameters TEXT,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at INTEGER,
    updated_at INTEGER
);

-- SubtitleGroup表
CREATE TABLE IF NOT EXISTS subtitle_group (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    last_update INTEGER,
    created_at INTEGER
);

-- UserSubscription表
CREATE TABLE IF NOT EXISTS user_subscriptions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    bangumi_id INTEGER NOT NULL, -- Rust端为i64，SQL NOT NULL
    subscribed_at INTEGER NOT NULL,
    notes TEXT,
    anime_name TEXT,
    anime_name_cn TEXT,
    anime_rating REAL,
    anime_air_date TEXT,
    anime_air_weekday INTEGER
);

CREATE INDEX IF NOT EXISTS idx_user_subscriptions_user_id ON user_subscriptions (user_id);
CREATE INDEX IF NOT EXISTS idx_user_subscriptions_bangumi_id ON user_subscriptions (bangumi_id);