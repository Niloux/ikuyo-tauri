import { defineStore } from 'pinia'
import type { DownloadTask, ProgressUpdate } from '@/services/download/downloadTypes'
import { listen } from '@tauri-apps/api/event'
import { downloadApiService } from '@/services/download/downloadApiService'

// 解决 error_msg 冲突：去除 ProgressUpdate 的 error_msg

type ProgressUpdateWithoutError = Omit<ProgressUpdate, 'error_msg'>

interface DownloadTaskState extends DownloadTask, ProgressUpdateWithoutError { }

interface DownloadStoreState {
    tasks: Record<number, DownloadTaskState>
}

let hasInit = false // 防止重复注册
let lastMissingProgress: import('@/services/download/downloadTypes').ProgressUpdate | null = null
let fetchDebounceTimer: ReturnType<typeof setTimeout> | null = null

export const useDownloadStore = defineStore('download', {
    state: (): DownloadStoreState => ({
        tasks: {},
    }),
    actions: {
        async init() {
            if (hasInit) return
            hasInit = true
            await listen<import('@/services/download/downloadTypes').ProgressUpdate>('download_progress', (event) => {
                console.log('收到进度更新', event.payload)
                this.updateProgress(event.payload)
            })
        },
        async fetchAllDownloads() {
            const list = await downloadApiService.fetchAllDownloads()
            const tasks: Record<number, DownloadTaskState> = {}
            for (const task of list) {
                tasks[task.id] = {
                    ...task,
                    total_bytes: 0,
                    progress: 0,
                    speed: 0,
                    time_remaining: '',
                }
            }
            this.tasks = tasks
            // fetch 后尝试补合并丢失的 progress
            if (lastMissingProgress && tasks[lastMissingProgress.id]) {
                this.updateProgress(lastMissingProgress)
                lastMissingProgress = null
            }
        },
        updateProgress(progress: import('@/services/download/downloadTypes').ProgressUpdate) {
            const id = progress.id
            if (!this.tasks[id]) {
                // 防抖触发 fetchAllDownloads
                lastMissingProgress = progress
                if (!fetchDebounceTimer) {
                    fetchDebounceTimer = setTimeout(() => {
                        this.fetchAllDownloads()
                        fetchDebounceTimer = null
                    }, 500)
                }
                return
            }
            // 合并进度信息，error_msg 直接覆盖
            this.tasks[id] = {
                ...this.tasks[id],
                ...progress,
                error_msg: progress.error_msg,
            }
        },
        async startDownload(task: import('@/services/download/downloadTypes').StartDownloadTask) {
            await downloadApiService.startDownload(task)
            await this.fetchAllDownloads()
        },
        async pauseDownload(id: number) {
            await downloadApiService.pauseDownload(id)
            await this.fetchAllDownloads()
        },
        async resumeDownload(id: number) {
            await downloadApiService.resumeDownload(id)
            await this.fetchAllDownloads()
        },
        async removeDownload(id: number, delete_files: boolean) {
            await downloadApiService.removeDownload(id, delete_files)
            await this.fetchAllDownloads()
        },
    },
    getters: {
        // 获取所有任务列表
        getTaskList(state): DownloadTaskState[] {
            return Object.values(state.tasks)
        },
        // 获取所有出错任务
        getErrorTasks(state): DownloadTaskState[] {
            return Object.values(state.tasks).filter(t => t.error_msg)
        },
        getTaskByResourceId: (state) => (resourceId: number) => {
            return Object.values(state.tasks).find(task => task.resource_id === resourceId)
        },
    },
})

// API 操作失败时建议由调用方（如页面/组件）捕获异常并通过全局 toast/notification 反馈用户 