<template>
  <div class="task-management-view">
    <!-- 采集任务区域 -->
    <div class="task-section">
      <div class="section-header">
        <span class="section-title">采集任务</span>
        <button @click="openCreateTaskModal" class="create-button">新建任务</button>
      </div>
      <div class="section-content">
        <TaskTable
          :tasks="taskStore.tasks"
          :isLoading="taskStore.fetchTasksAsync.loading"
          :error="taskStore.fetchTasksAsync.error"
          :onCancel="cancelTask"
          :currentPage="taskStore.currentPage"
          :pageSize="taskStore.pageSize"
          @retry="taskStore.fetchTasks"
          @page-change="handlePageChange"
        />
      </div>
    </div>

    <!-- 任务创建模态框 -->
    <TaskModal
      :visible="showCreateTaskModal"
      :task="newTask"
      :errors="createTaskFormErrors"
      :onSubmit="submitCreateTask"
      :onCancel="closeCreateTaskModal"
      :onUpdateTask="(t: CrawlerTaskCreate) => { newTask = t }"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, onActivated } from 'vue'
import { useTaskStore } from '../stores/taskStore'
import { useFeedbackStore } from '../stores/feedbackStore'
import { useErrorHandler } from '../utils/useErrorHandler'
import TaskTable from '../components/TaskTable.vue'
import { defineAsyncComponent } from 'vue'
import type { CrawlerTaskCreate } from '../services/crawler/crawlerTypes'
import { ensureScrollToTop } from '../utils/scrollUtils'

const taskStore = useTaskStore()
const { handleApiError, showSuccess, handleValidationErrors } = useErrorHandler()

const showCreateTaskModal = ref(false)

// 新任务表单数据
const newTask = ref<CrawlerTaskCreate>({
  mode: 'homepage',
  year: undefined,
  season: undefined,
  limit: undefined,
})

// 监听进行中任务，自动开启轮询
watch(
  () => (Array.isArray(taskStore.tasks) ? taskStore.tasks : []).map(t => ({ id: t.id, status: t.status })),
  (newTasks) => {
    newTasks.forEach(task => {
      if (["pending", "running"].includes(task.status)) {
        taskStore.startTaskPolling(task.id)
      } else {
        taskStore.stopTaskPolling(task.id)
      }
    })
  },
  { immediate: true, deep: true }
)

onMounted(() => {
  ensureScrollToTop()
  taskStore.fetchTasks()
})

onActivated(() => {
  ensureScrollToTop()
})

onUnmounted(() => {
  taskStore.stopAllTaskPolling()
})

const openCreateTaskModal = () => {
  newTask.value = {
    mode: 'homepage',
    year: undefined,
    season: undefined,
    limit: undefined,
  }
  showCreateTaskModal.value = true
}

const closeCreateTaskModal = () => {
  showCreateTaskModal.value = false
}

const createTaskFormErrors = ref<{ [key: string]: string }>({})

function validateCreateTaskForm() {
  createTaskFormErrors.value = {}
  if (!newTask.value.mode) {
    createTaskFormErrors.value.mode = '请选择模式'
  }
  if ((newTask.value.mode === 'season' || newTask.value.mode === 'year') && !newTask.value.year) {
    createTaskFormErrors.value.year = '请选择年份'
  }
  if (newTask.value.mode === 'season' && !newTask.value.season) {
    createTaskFormErrors.value.season = '请选择季度'
  }
  if (newTask.value.limit !== undefined && newTask.value.limit !== null && (!Number.isInteger(newTask.value.limit) || newTask.value.limit <= 0)) {
    createTaskFormErrors.value.limit = '请输入正整数或留空'
  }
  return Object.keys(createTaskFormErrors.value).length === 0
}

const submitCreateTask = async () => {
  if (!validateCreateTaskForm()) return
  try {
    const payload: CrawlerTaskCreate = {
      mode: newTask.value.mode,
      limit: newTask.value.limit,
    }
    if (newTask.value.mode === 'season' || newTask.value.mode === 'year') {
      payload.year = newTask.value.year
    }
    if (newTask.value.mode === 'season') {
      payload.season = newTask.value.season
    }
    const createdTask = await taskStore.createTask(payload)
    closeCreateTaskModal()
    // 直接开启轮询
    taskStore.startTaskPolling(createdTask.id)
  } catch (e: any) {
    handleApiError(e, '创建任务失败')
  }
}

const cancelTask = async (taskId: number) => {
  try {
    await ElMessageBox.confirm(
      '确定要取消这个任务吗？',
      '提示',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning',
      }
    )
    // 用户点击了“确定”
    const cancelledTask = await taskStore.cancelTask(taskId)
    showSuccess('任务已取消！')
    taskStore.stopTaskPolling(taskId)
  } catch (e: any) {
    // 用户点击了“取消”或关闭弹窗，不做任何事
    if (e !== 'cancel' && e !== 'close') {
      handleApiError(e, '取消任务失败')
    }
  }
}

const handlePageChange = (page: number) => {
  taskStore.currentPage = page
  taskStore.fetchTasks()
}

const TaskModal = defineAsyncComponent(() => import('../components/TaskModal.vue'))
</script>

<style src="../styles/globals.css"></style>
