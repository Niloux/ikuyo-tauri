/**
 * 下载API服务
 * 封装所有下载相关的后端API调用
 */

import { invoke } from '@tauri-apps/api/core'
import type { StartDownloadTask, DownloadTask } from './downloadTypes'

class DownloadApiService {
    /**
     * 添加下载任务
     */
    async startDownload(task: StartDownloadTask): Promise<number> {
        return invoke('start_download', { task })
    }

    /**
     * 获取下载任务列表
     */
    async fetchAllDownloads(): Promise<DownloadTask[]> {
        return invoke('list_downloads')
    }

    /**
     * 暂停下载任务
     */
    async pauseDownload(id: number): Promise<void> {
        return invoke('pause_download', { id })
    }

    /**
     * 恢复下载任务
     */
    async resumeDownload(id: number): Promise<void> {
        return invoke('resume_download', { id })
    }

    /**
     * 删除下载任务
     */
    async removeDownload(id: number, delete_files: boolean): Promise<void> {
        return invoke('remove_download', { id, delete_files })
    }

    /**
     * 获取下载任务路径
     */
    async getDownloadPath(id: number): Promise<string> {
        return invoke('get_download_path', { id })
    }
}

export const downloadApiService = new DownloadApiService()