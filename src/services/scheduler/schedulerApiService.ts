// =============================================================================
// Scheduler API Service
// =============================================================================

import { invoke } from '@tauri-apps/api/core';
import type { ScheduledJobCreate, ScheduledJobUpdate, ScheduledJobResponse } from './schedulerTypes';

export class ScheduledJobApiService {
    /**
     * 获取所有计划任务列表
     */
    static async listScheduledJobs(): Promise<ScheduledJobResponse[]> {
        return await invoke('get_scheduled_jobs');
    }

    /**
     * 创建新的计划任务
     * @param data 计划任务创建数据
     */
    static async createScheduledJob(data: ScheduledJobCreate):
        Promise<ScheduledJobResponse> {
        return await invoke('create_scheduled_job', { job: data });
    }

    /**
     * 获取特定计划任务详情
     * @param job_id 计划任务的job_id
     */
    static async getScheduledJob(job_id: string): Promise<ScheduledJobResponse> {
        return await invoke('get_scheduled_job', { job_id });
    }

    /**
     * 更新特定计划任务
     * @param job_id 计划任务的job_id
     * @param data 计划任务更新数据
     */
    static async updateScheduledJob(job_id: string, data: ScheduledJobUpdate):
        Promise<ScheduledJobResponse> {
        return await invoke('update_scheduled_job', { job_id, updates: data });
    }

    /**
     * 删除特定计划任务
     * @param job_id 计划任务的job_id
     */
    static async deleteScheduledJob(job_id: string): Promise<void> {
        return await invoke('delete_scheduled_job', { job_id });
    }

    /**
     * 切换特定计划任务的启用/禁用状态
     * @param job_id 计划任务的job_id
     */
    static async toggleScheduledJob(job_id: string):
        Promise<ScheduledJobResponse> {
        return await invoke('toggle_scheduled_job', { job_id });
    }
}

export default ScheduledJobApiService;
