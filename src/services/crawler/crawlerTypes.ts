// =============================================================================
// Crawler Types
// =============================================================================

export interface CrawlerTaskCreate {
    mode: 'homepage' | 'season' | 'year';
    year?: number;
    season?: '春' | '夏' | '秋' | '冬';
    limit?: number;
}

export type CrawlerTaskType = 'manual' | 'schedule';
export type CrawlerTaskStatus = 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';

// 爬虫任务相关类型
export interface TaskResponse {
    id: number;
    task_type: CrawlerTaskType;
    status: CrawlerTaskStatus;
    parameters?: string;
    result_summary?: string;
    created_at?: number; // Unix timestamp in milliseconds
    started_at?: number;
    completed_at?: number;
    error_message?: string;
    percentage?: number;
    processed_items?: number;
    total_items?: number;
    processing_speed?: number;
    estimated_remaining?: number;
}
