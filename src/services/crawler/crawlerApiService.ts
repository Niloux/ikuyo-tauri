// =============================================================================
// Crawler API Service
// =============================================================================

import { invoke } from '@tauri-apps/api/core';
import type { CrawlerTaskCreate, TaskResponse } from './crawlerTypes';

export class CrawlerApiService {
    /**
     * 创建新的爬虫任务
     * @param data 任务创建数据
     */
    static async createTask(data: CrawlerTaskCreate): Promise<TaskResponse> {
        return await invoke('create_crawler_task', { task: data });
    }

    /**
     * 获取所有爬虫任务列表
     * @param page 页码，从1开始
     * @param pageSize 每页数量
     */
    static async listTasks(page: number = 1, pageSize: number = 6):
        Promise<TaskResponse[]> {
        return await invoke('list_crawler_tasks', {
            page,
            page_size: pageSize
        });
    }

    /**
     * 获取特定爬虫任务的详情
     * @param taskId 任务ID
     */
    static async getTask(taskId: number): Promise<TaskResponse> {
        return await invoke('get_crawler_task', { task_id: taskId });
    }

    /**
     * 取消特定爬虫任务
     * @param taskId 任务ID
     */
    static async cancelTask(taskId: number): Promise<TaskResponse> {
        return await invoke('cancel_crawler_task', { task_id: taskId });
    }

    /**
     * 删除特定爬虫任务
     * @param taskId 任务ID
     */
    static async deleteTask(taskId: number): Promise<void> {
        return await invoke('delete_crawler_task', { task_id: taskId });
    }

    /**
     * 获取特定爬虫任务的进度状态
     * @param taskId 任务ID
     */
    static async getTaskProgress(taskId: number): Promise<TaskResponse> {
        return await invoke('get_crawler_task_status', { task_id: taskId });
    }

    // 注意：WebSocket功能在Tauri中可以用事件系统替代
    // 如果需要实时更新，建议使用Tauri的事件监听机制
}

export default CrawlerApiService;
