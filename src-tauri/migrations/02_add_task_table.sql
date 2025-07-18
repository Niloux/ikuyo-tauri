-- 02_add_task_table.sql
CREATE TABLE IF NOT EXISTS download_task (
    id INTEGER PRIMARY KEY,
    magnet_url TEXT NOT NULL,
    save_path TEXT,
    status TEXT NOT NULL,
    bangumi_id INTEGER NOT NULL,
    resource_id INTEGER NOT NULL,
    episode_number INTEGER NOT NULL,
    name TEXT NOT NULL,
    name_cn TEXT NOT NULL,
    cover TEXT NOT NULL,
    total_size INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    error_msg TEXT
);

CREATE INDEX IF NOT EXISTS idx_download_task_bangumi_id ON download_task(bangumi_id);
CREATE INDEX IF NOT EXISTS idx_download_task_resource_id ON download_task(resource_id);
CREATE INDEX IF NOT EXISTS idx_download_task_episode_number ON download_task(episode_number); 