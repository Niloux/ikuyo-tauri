import { defineStore } from "pinia";
import { ref } from "vue";
import { useAsyncAction } from "./asyncUtils";

import CrawlerApiService from "../services/crawler/crawlerApiService";
import type {
  CrawlerTaskCreate,
  TaskResponse,
} from "../services/crawler/crawlerTypes";

/**
 * Pinia 采集任务管理 Store
 * 专注管理采集任务相关状态、异步操作和轮询进度
 * 定时任务请使用 schedulerStore
 */
export const useTaskStore = defineStore("task", () => {
  const tasks = ref<TaskResponse[]>([]);
  const currentPage = ref(1);
  const pageSize = ref(6);
  // 轮询定时器map
  const pollingTimers = new Map<number, number>();

  // --- 采集任务相关操作 ---
  /**
   * 获取所有采集任务列表
   * @returns Promise<TaskResponse[]>
   */
  const fetchTasksAsync = useAsyncAction(() =>
    CrawlerApiService.listTasks(currentPage.value, pageSize.value),
  );
  const fetchTasks = async () => {
    const result = await fetchTasksAsync.run();
    tasks.value = result;
    return result;
  };

  /**
   * 创建新的采集任务
   * @param taskCreateData 任务创建数据
   * @returns Promise<TaskResponse>
   */
  const createTaskAsync = useAsyncAction((taskCreateData: CrawlerTaskCreate) =>
    CrawlerApiService.createTask(taskCreateData),
  );
  const createTask = async (taskCreateData: CrawlerTaskCreate) => {
    const result = await createTaskAsync.run(taskCreateData);
    await fetchTasks();
    startTaskPolling(result.id);
    return result;
  };

  /**
   * 取消采集任务
   * @param taskId 任务ID
   * @returns Promise<TaskResponse>
   */
  const cancelTaskAsync = useAsyncAction((taskId: number) =>
    CrawlerApiService.cancelTask(taskId),
  );
  const cancelTask = async (taskId: number) => {
    const result = await cancelTaskAsync.run(taskId);
    const index = tasks.value.findIndex((t) => t.id === taskId);
    if (index !== -1) {
      tasks.value[index] = result;
    }
    stopTaskPolling(taskId);
    return result;
  };

  // --- 轮询相关 ---
  const startTaskPolling = (taskId: number) => {
    if (pollingTimers.has(taskId)) return;
    const timer = window.setInterval(async () => {
      const task = await CrawlerApiService.getTaskProgress(taskId);
      const index = tasks.value.findIndex((t) => t.id === taskId);
      if (index !== -1) {
        tasks.value[index] = task;
      }
      if (["completed", "failed", "cancelled"].includes(task.status)) {
        stopTaskPolling(taskId);
      }
    }, 1000);
    pollingTimers.set(taskId, timer);
  };

  const stopTaskPolling = (taskId: number) => {
    const timer = pollingTimers.get(taskId);
    if (timer) {
      clearInterval(timer);
      pollingTimers.delete(taskId);
    }
  };

  const stopAllTaskPolling = () => {
    for (const timer of pollingTimers.values()) {
      clearInterval(timer);
    }
    pollingTimers.clear();
  };

  return {
    // 状态
    tasks,
    currentPage,
    pageSize,

    // 采集任务操作
    fetchTasks,
    createTask,
    cancelTask,

    // 异步状态
    fetchTasksAsync,
    createTaskAsync,
    cancelTaskAsync,

    // 轮询操作
    startTaskPolling,
    stopTaskPolling,
    stopAllTaskPolling,
  };
});
