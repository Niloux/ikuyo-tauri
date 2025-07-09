<template>
  <div class="task-management-view">
    <!-- 即时任务区域 -->
    <div class="task-section">
      <div class="section-header">
        <span class="section-title">即时任务</span>
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

    <!-- 定时任务区域 -->
    <div class="task-section">
      <div class="section-header">
        <span class="section-title">定时任务</span>
        <button @click="openCreateScheduledJobModal" class="create-button">新建定时任务</button>
      </div>
      <div class="section-content">
        <ScheduledJobTable
          :jobs="schedulerStore.scheduledJobs"
          :isLoading="schedulerStore.fetchScheduledJobsAsync.loading"
          :error="schedulerStore.fetchScheduledJobsAsync.error"
          :onEdit="editScheduledJob"
          :onDelete="deleteScheduledJob"
          :onToggle="toggleScheduledJob"
          @retry="schedulerStore.fetchScheduledJobs"
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

    <!-- 定时任务模态框 -->
    <ScheduledJobModal
      :visible="showScheduledJobModal"
      :job="currentScheduledJob"
      :errors="scheduledJobFormErrors"
      :editing="!!editingJob"
      :onSubmit="submitScheduledJob"
      :onCancel="closeScheduledJobModal"
      :onUpdateJob="updateScheduledJob"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, onActivated } from 'vue'
import { useTaskStore } from '../stores/taskStore'
import { useSchedulerStore } from '../stores/schedulerStore'
import { useFeedbackStore } from '../stores/feedbackStore'
import { useErrorHandler } from '../utils/useErrorHandler'
import TaskTable from '../components/TaskTable.vue'
import ScheduledJobTable from '../components/ScheduledJobTable.vue'
import { defineAsyncComponent } from 'vue'
import type { CrawlerTaskCreate } from '../services/crawler/crawlerTypes'
import type { ScheduledJobCreate, ScheduledJobUpdate, ScheduledJobResponse } from '../services/scheduler/schedulerTypes'
import { ensureScrollToTop } from '../utils/scrollUtils'
import { ElMessageBox } from 'element-plus'

const taskStore = useTaskStore()
const schedulerStore = useSchedulerStore()
const feedbackStore = useFeedbackStore()
const { handleApiError, showSuccess, handleValidationErrors } = useErrorHandler()

const showCreateTaskModal = ref(false)
const showScheduledJobModal = ref(false)
const editingJob = ref<ScheduledJobResponse | null>(null)

// 新任务表单数据
const newTask = ref<CrawlerTaskCreate>({
  mode: 'homepage',
  year: undefined,
  season: undefined,
  limit: undefined,
})

// 定时任务表单数据
const currentScheduledJob = ref<ScheduledJobCreate & { parameters_json?: string }>({
  job_id: '',
  name: '',
  cron_expression: '',
  parameters: {},
  parameters_json: '{}',
  enabled: true,
  description: '',
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
  schedulerStore.fetchScheduledJobs()
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
const scheduledJobFormErrors = ref<{ [key: string]: string }>({})

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

function validateScheduledJobForm() {
  scheduledJobFormErrors.value = {}
  if (!currentScheduledJob.value.job_id) {
    scheduledJobFormErrors.value.job_id = '请输入任务ID'
  }
  if (!currentScheduledJob.value.name) {
    scheduledJobFormErrors.value.name = '请输入名称'
  }
  if (!currentScheduledJob.value.cron_expression) {
    scheduledJobFormErrors.value.cron_expression = '请输入Cron表达式'
  }
  if (currentScheduledJob.value.parameters_json) {
    try {
      JSON.parse(currentScheduledJob.value.parameters_json)
    } catch {
      scheduledJobFormErrors.value.parameters_json = '参数需为合法JSON'
    }
  }
  return Object.keys(scheduledJobFormErrors.value).length === 0
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

const openCreateScheduledJobModal = () => {
  editingJob.value = null
  currentScheduledJob.value = {
    job_id: '',
    name: '',
    cron_expression: '',
    parameters: {},
    parameters_json: '{}',
    enabled: true,
    description: '',
  }
  showScheduledJobModal.value = true
}

const closeScheduledJobModal = () => {
  showScheduledJobModal.value = false
}

const updateScheduledJob = (j: ScheduledJobCreate & { parameters_json?: string }) => {
  currentScheduledJob.value = j
}

const editScheduledJob = (job: ScheduledJobResponse) => {
  editingJob.value = job
  currentScheduledJob.value = {
    job_id: job.job_id,
    name: job.name,
    cron_expression: job.cron_expression,
    parameters: job.parameters,
    parameters_json: JSON.stringify(job.parameters, null, 2),
    enabled: job.enabled,
    description: job.description || '',
  }
  showScheduledJobModal.value = true
}

const submitScheduledJob = async () => {
  if (!validateScheduledJobForm()) return
  try {
    let parsedParameters = {}
    if (currentScheduledJob.value.parameters_json) {
      try {
        parsedParameters = JSON.parse(currentScheduledJob.value.parameters_json)
      } catch (e) {
        handleApiError('参数JSON格式不正确！')
        return
      }
    }
    const payload = {
      job_id: currentScheduledJob.value.job_id,
      name: currentScheduledJob.value.name,
      cron_expression: currentScheduledJob.value.cron_expression,
      parameters: parsedParameters,
      enabled: currentScheduledJob.value.enabled,
      description: currentScheduledJob.value.description,
    }
    if (editingJob.value) {
      await schedulerStore.updateScheduledJob(editingJob.value.job_id, payload as ScheduledJobUpdate)
      showSuccess('定时任务更新成功！')
    } else {
      await schedulerStore.createScheduledJob(payload as ScheduledJobCreate)
      showSuccess('定时任务创建成功！')
    }
    closeScheduledJobModal()
  } catch (e: any) {
    handleApiError(e, '操作失败')
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

const toggleScheduledJob = async (jobId: string) => {
  try {
    await schedulerStore.toggleScheduledJob(jobId)
    showSuccess('定时任务状态已更新！')
  } catch (e: any) {
    handleApiError(e, '更新定时任务状态失败')
  }
}

const deleteScheduledJob = async (jobId: string) => {
  try {
    await ElMessageBox.confirm(
      '确定要删除这个定时任务吗？',
      '提示',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning',
      }
    )
    await schedulerStore.deleteScheduledJob(jobId)
    showSuccess('定时任务已删除！')
  } catch (e: any) {
    if (e !== 'cancel' && e !== 'close') {
      handleApiError(e, '删除定时任务失败')
    }
  }
}

const handlePageChange = (page: number) => {
  taskStore.currentPage = page
  taskStore.fetchTasks()
}

const TaskModal = defineAsyncComponent(() => import('../components/TaskModal.vue'))
const ScheduledJobModal = defineAsyncComponent(() => import('../components/ScheduledJobModal.vue'))
</script>

<style src="../assets/task.css"></style>
