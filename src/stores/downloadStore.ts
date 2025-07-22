import { defineStore } from 'pinia'
import type { DownloadTask, ProgressUpdate } from '@/services/download/downloadTypes'
import { listen } from '@tauri-apps/api/event'
import { downloadApiService } from '@/services/download/downloadApiService'

type ProgressUpdateWithoutError = Omit<ProgressUpdate, 'error_msg'>

// DownloadTaskState 必须包含 resource_id 字段，确保 resource 与任务一一对应
interface DownloadTaskState extends DownloadTask, ProgressUpdateWithoutError { }

interface DownloadStoreState {
    tasks: Record<number, DownloadTaskState>,
    resourceIdToTaskId: Record<number, number>,
}

let hasInit = false // 防止重复注册
let lastMissingProgress: import('@/services/download/downloadTypes').ProgressUpdate | null = null
let fetchDebounceTimer: ReturnType<typeof setTimeout> | null = null

export const useDownloadStore = defineStore('download', {
    state: (): DownloadStoreState => ({
        tasks: {},
        resourceIdToTaskId: {},
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
            const resourceIdToTaskId: Record<number, number> = {}
            for (const task of list) {
                tasks[task.id] = {
                    ...task,
                    total_bytes: 0,
                    progress: 0,
                    speed: 0,
                    time_remaining: '',
                }
                if (typeof task.resource_id === 'number') {
                    resourceIdToTaskId[task.resource_id] = task.id
                }
            }
            this.tasks = tasks
            this.resourceIdToTaskId = resourceIdToTaskId
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
            // 同步 resourceIdToTaskId
            if (typeof this.tasks[id].resource_id === 'number') {
                this.resourceIdToTaskId[this.tasks[id].resource_id] = id
            }
        },
        async startDownload(task: import('@/services/download/downloadTypes').StartDownloadTask) {
            const newTaskId = await downloadApiService.startDownload(task)
            // 先插入一个初始任务对象，等待进度事件推送后再补全
            this.tasks[newTaskId] = {
                ...task,
                id: newTaskId,
                status: 'pending',
                total_bytes: task.total_size,
                progress: 0,
                speed: 0,
                time_remaining: '',
                error_msg: null,
                created_at: Date.now(),
                updated_at: Date.now(),
                save_path: task.save_path || '',
            }
            this.resourceIdToTaskId[task.resource_id] = newTaskId
        },
        async pauseDownload(id: number) {
            await downloadApiService.pauseDownload(id)
            if (this.tasks[id]) {
                this.tasks[id].status = 'paused'
                this.tasks[id].updated_at = Date.now()
            }
        },
        async resumeDownload(id: number) {
            await downloadApiService.resumeDownload(id)
            if (this.tasks[id]) {
                this.tasks[id].status = 'downloading'
                this.tasks[id].updated_at = Date.now()
            }
        },
        async removeDownload(id: number, delete_files: boolean) {
            await downloadApiService.removeDownload(id, delete_files)
            // 移除本地任务
            const task = this.tasks[id]
            if (task) {
                delete this.resourceIdToTaskId[task.resource_id]
                delete this.tasks[id]
            }
        },
        async getDownloadPath(id: number) {
            return await downloadApiService.getDownloadPath(id)
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
            const taskId = state.resourceIdToTaskId[resourceId]
            if (typeof taskId === 'number') {
                return state.tasks[taskId]
            }
            return undefined
        },
        // 新增：返回 DownloadButton 所需全部 UI 状态
        getTaskUIState: (state) => (resourceId: number) => {
            const task = (state as any).getTaskByResourceId(resourceId)
            if (!task) {
                return {
                    status: null,
                    progress: 0,
                    errorMsg: null,
                    buttonText: '下载',
                    disabled: false,
                    speed: undefined,
                    timeRemaining: undefined,
                    taskId: undefined,
                }
            }
            let buttonText = '下载'
            switch (task.status) {
                case 'downloading': buttonText = '下载中'; break
                case 'completed': buttonText = '已下载'; break
                case 'failed': buttonText = '重试'; break
                case 'paused': buttonText = '已暂停'; break
                case 'pending': buttonText = '等待中'; break
                default: buttonText = '下载'; break
            }
            return {
                status: task.status,
                progress: task.progress || 0,
                errorMsg: task.error_msg,
                buttonText,
                disabled: false,
                speed: task.speed,
                timeRemaining: task.time_remaining,
                taskId: task.id,
            }
        },
    },
})

// API 操作失败时建议由调用方（如页面/组件）捕获异常并通过全局 toast/notification 反馈用户 