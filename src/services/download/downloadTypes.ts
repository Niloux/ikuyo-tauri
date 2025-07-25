/**
 * 下载功能相关类型定义
 */

// 发起下载任务参数
export interface StartDownloadTask {
  magnet_url: string;
  save_path?: string;
  title: string;
  bangumi_id: number;
  resource_id: number;
  episode_number: number;
  name: string;
  name_cn: string;
  cover: string;
  total_size: number;
}

// 下载事件结构体
export interface ProgressUpdate {
  id: number;
  total_bytes: number;
  progress: number;
  speed: number;
  time_remaining: string;
  status:
    | "pending"
    | "downloading"
    | "paused"
    | "completed"
    | "failed"
    | "deleted";
  error_msg: string | null;
}

// 下载任务列表
export interface DownloadTask {
  id: number;
  magnet_url: string;
  save_path: string;
  status:
    | "pending"
    | "downloading"
    | "paused"
    | "completed"
    | "failed"
    | "deleted";
  title: string;
  bangumi_id: number;
  resource_id: number;
  episode_number: number;
  name: string;
  name_cn: string;
  cover: string;
  total_size: number;
  created_at: number;
  updated_at: number;
  error_msg: string | null;
}
