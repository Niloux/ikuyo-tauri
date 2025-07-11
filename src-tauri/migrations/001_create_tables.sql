-- Anime表
CREATE TABLE IF NOT EXISTS anime (
    mikan_id INTEGER PRIMARY KEY,
    bangumi_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    original_title TEXT,
    broadcast_day TEXT,
    broadcast_start TEXT,
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
    anime_air_weekday INTEGER,
    -- 新增字段
    url TEXT,
    item_type INTEGER,
    summary TEXT,
    rank INTEGER,
    images TEXT
);

CREATE INDEX IF NOT EXISTS idx_user_subscriptions_user_id ON user_subscriptions (user_id);
CREATE INDEX IF NOT EXISTS idx_user_subscriptions_bangumi_id ON user_subscriptions (bangumi_id);

-- Resource表
CREATE TABLE IF NOT EXISTS resource (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    mikan_id INTEGER NOT NULL,
    subtitle_group_id INTEGER NOT NULL,
    episode_number INTEGER,
    title TEXT NOT NULL,
    file_size TEXT,
    resolution TEXT,
    subtitle_type TEXT,
    magnet_url TEXT,
    torrent_url TEXT,
    play_url TEXT,
    magnet_hash TEXT UNIQUE,
    release_date INTEGER,
    created_at INTEGER,
    updated_at INTEGER,
    FOREIGN KEY (mikan_id) REFERENCES anime (mikan_id),
    FOREIGN KEY (subtitle_group_id) REFERENCES subtitle_group (id)
);

CREATE INDEX IF NOT EXISTS idx_resource_mikan_id ON resource (mikan_id);
CREATE INDEX IF NOT EXISTS idx_resource_subtitle_group_id ON resource (subtitle_group_id);
CREATE INDEX IF NOT EXISTS idx_resource_episode_number ON resource (episode_number);
CREATE INDEX IF NOT EXISTS idx_resource_title ON resource (title);
CREATE INDEX IF NOT EXISTS idx_resource_resolution ON resource (resolution);
CREATE INDEX IF NOT EXISTS idx_resource_subtitle_type ON resource (subtitle_type);
CREATE INDEX IF NOT EXISTS idx_release_date_desc ON resource (release_date DESC);

-- Bangumi subject 缓存表
CREATE TABLE IF NOT EXISTS bangumi_subject_cache (
    id INTEGER PRIMARY KEY,
    content TEXT NOT NULL,
    updated_at INTEGER NOT NULL,
    ttl INTEGER NOT NULL
);

-- Bangumi episodes 缓存表
CREATE TABLE IF NOT EXISTS bangumi_episodes_cache (
    id INTEGER NOT NULL,
    params_hash TEXT NOT NULL,
    content TEXT NOT NULL,
    updated_at INTEGER NOT NULL,
    ttl INTEGER NOT NULL,
    PRIMARY KEY (id, params_hash)
);

-- Bangumi calendar 缓存表
CREATE TABLE IF NOT EXISTS bangumi_calendar_cache (
    id INTEGER PRIMARY KEY,
    content TEXT NOT NULL,
    updated_at INTEGER NOT NULL,
    ttl INTEGER NOT NULL
);
-- 以上为缓存相关表结构
