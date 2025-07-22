<template>
  <div class="download-management-view">
    <div class="content-card">
      <!-- 状态标签区域 -->
      <div class="status-tabs">
        <div 
          v-for="tab in statusTabs" 
          :key="tab.key"
          :class="['tab-item', { active: activeTab === tab.key }]"
          @click="setActiveTab(tab.key)"
        >
          {{ tab.label }}
          <span v-if="tab.count > 0" class="tab-count">{{ tab.count }}</span>
        </div>
        <div class="tab-actions">
          <button class="action-link" @click="openDownloadFolder">
            📂 打开目录
          </button>
        </div>
      </div>

      <!-- 操作栏 -->
      <div class="action-bar">
        <div class="action-left">
          <button 
            v-if="hasActiveDownloads" 
            class="action-btn primary" 
            @click="pauseAllDownloads"
          >
            ⏸️ 暂停全部
          </button>
          <button 
            v-if="hasPausedDownloads" 
            class="action-btn primary" 
            @click="resumeAllDownloads"
          >
            ▶️ 继续全部
          </button>
          <button class="action-btn secondary" @click="showMoreActions = !showMoreActions">
            ⋯
          </button>
          <div v-if="showMoreActions" class="more-actions-dropdown">
            <button @click="clearCompleted">清空已完成</button>
            <button @click="retryAllFailed">重试失败任务</button>
          </div>
        </div>
        <div class="action-right">
          <div class="search-box">
            <input 
              v-model="searchQuery"
              type="text" 
              placeholder="搜索番剧名称..."
              class="search-input"
            />
            <span class="search-icon">🔍</span>
          </div>
        </div>
      </div>

      <!-- 表格头部 -->
      <div class="table-header">
        <div class="col col-index">#</div>
        <div class="col col-title">标题</div>
        <div class="col col-status">状态</div>
        <div class="col col-time">下载时间</div>
        <div class="col col-size">大小</div>
        <div class="col col-actions">操作</div>
      </div>

      <!-- 任务列表 -->
      <div class="task-list">
        <!-- 加载状态 -->
        <div v-if="loading" class="loading-state">
          <div class="loading-spinner"></div>
          <p>加载中...</p>
        </div>

        <!-- 任务行 -->
        <div 
          v-else-if="filteredTasks.length > 0"
          v-for="(task, index) in filteredTasks" 
          :key="task.id"
          :class="['task-row', `status-${task.status}`]"
          @contextmenu.prevent="showContextMenu($event, task)"
        >
          <div class="col col-index">{{ String(index + 1).padStart(2, '0') }}</div>
          <div class="col col-title">
            <div class="task-info">
              <img 
                :src="task.cover || defaultCover" 
                :alt="task.name_cn || task.name"
                class="task-cover"
                @error="onImageError"
              />
              <div class="task-details">
                <div class="task-name">{{ task.name_cn || task.name }}</div>
                <div class="task-episode">第{{ task.episode_number }}集</div>
              </div>
            </div>
          </div>
          <div class="col col-status">
            <div class="status-content">
              <div v-if="task.status === 'downloading'" class="downloading-status">
                <div class="progress-bar">
                  <div 
                    class="progress-fill" 
                    :style="{ width: `${Math.round((task.progress || 0) * 100)}%` }"
                  ></div>
                </div>
                <div class="progress-text">
                  {{ Math.round((task.progress || 0) * 100) }}% | {{ formatSpeed(task.speed) }}
                </div>
              </div>
              <div v-else-if="task.status === 'completed'" class="completed-status">
                <span class="status-icon">✅</span>
                <span>已完成</span>
              </div>
              <div v-else-if="task.status === 'paused'" class="paused-status">
                <span class="status-icon">⏸️</span>
                <span>已暂停</span>
              </div>
              <div v-else-if="task.status === 'failed'" class="failed-status">
                <span class="status-icon">❌</span>
                <span>下载失败</span>
                <button class="retry-btn" @click="retryDownload(task.id)">重试</button>
              </div>
              <div v-else class="pending-status">
                <span class="status-icon">⏳</span>
                <span>等待中</span>
              </div>
            </div>
          </div>
          <div class="col col-time">
            {{ formatTime(task.updated_at) }}
          </div>
          <div class="col col-size">
            {{ formatSize(task.total_bytes || task.total_size) }}
          </div>
          <div class="col col-actions">
            <div class="action-buttons">
              <button 
                v-if="task.status === 'downloading'"
                class="action-icon"
                @click="pauseDownload(task.id)"
                title="暂停"
              >
                ⏸️
              </button>
              <button 
                v-else-if="task.status === 'paused'"
                class="action-icon"
                @click="resumeDownload(task.id)"
                title="继续"
              >
                ▶️
              </button>
              <button 
                v-if="task.status === 'completed'"
                class="action-icon"
                @click="openFile(task.id)"
                title="打开文件"
              >
                📁
              </button>
              <button 
                class="action-icon delete"
                @click="deleteDownload(task.id)"
                title="删除"
              >
                🗑️
              </button>
            </div>
          </div>
        </div>

        <!-- 空状态 -->
        <div v-else class="empty-state">
          <div class="empty-icon">🎞</div>
          <h3>{{ getEmptyStateTitle() }}</h3>
          <p>{{ getEmptyStateMessage() }}</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { useDownloadStore } from '@/stores/downloadStore'
import { useFeedbackStore } from '@/stores/feedbackStore'
import { storeToRefs } from 'pinia'
import defaultCover from '@/assets/ikuyo-avatar.png'

const downloadStore = useDownloadStore()
const feedbackStore = useFeedbackStore()
const { tasks } = storeToRefs(downloadStore)

// 组件状态
const activeTab = ref<'all' | 'downloading' | 'completed' | 'paused' | 'failed'>('all')
const searchQuery = ref('')
const showMoreActions = ref(false)
const loading = ref(false)

// 状态标签配置
const statusTabs = computed(() => [
  { key: 'all' as const, label: '全部', count: tasks.value ? Object.keys(tasks.value).length : 0 },
  { key: 'downloading' as const, label: '正在下载', count: downloadingTasks.value.length },
  { key: 'completed' as const, label: '已完成', count: completedTasks.value.length },
  { key: 'paused' as const, label: '已暂停', count: pausedTasks.value.length },
  { key: 'failed' as const, label: '失败', count: failedTasks.value.length }
])

// 任务分类
const allTasks = computed(() => downloadStore.getTaskList)
const downloadingTasks = computed(() => allTasks.value.filter(task => task.status === 'downloading'))
const completedTasks = computed(() => allTasks.value.filter(task => task.status === 'completed'))
const pausedTasks = computed(() => allTasks.value.filter(task => task.status === 'paused'))
const failedTasks = computed(() => allTasks.value.filter(task => task.status === 'failed'))

// 筛选后的任务
const filteredTasks = computed(() => {
  let tasks = allTasks.value
  
  // 按状态筛选
  if (activeTab.value !== 'all') {
    tasks = tasks.filter(task => task.status === activeTab.value)
  }
  
  // 按搜索关键词筛选
  if (searchQuery.value.trim()) {
    const query = searchQuery.value.toLowerCase()
    tasks = tasks.filter(task => 
      (task.name && task.name.toLowerCase().includes(query)) ||
      (task.name_cn && task.name_cn.toLowerCase().includes(query))
    )
  }
  
  return tasks
})

// 状态检查
const hasActiveDownloads = computed(() => downloadingTasks.value.length > 0)
const hasPausedDownloads = computed(() => pausedTasks.value.length > 0)

// 方法
const setActiveTab = (tabKey: typeof activeTab.value) => {
  activeTab.value = tabKey
}

const formatSpeed = (speed: number): string => {
  if (!speed) return '0 MB/s'
  return `${speed.toFixed(2)} MB/s`
}

const formatSize = (bytes: number): string => {
  if (!bytes) return '0 B'
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`
}

const formatTime = (timestamp: number): string => {
  if (!timestamp) return '未知'
  const now = Date.now()
  const diff = now - timestamp * 1000
  
  if (diff < 60000) return '刚刚'
  if (diff < 3600000) return `${Math.floor(diff / 60000)}分钟前`
  if (diff < 86400000) return `${Math.floor(diff / 3600000)}小时前`
  return `${Math.floor(diff / 86400000)}天前`
}

const onImageError = (event: Event) => {
  const img = event.target as HTMLImageElement
  img.src = defaultCover
}

const getEmptyStateTitle = (): string => {
  if (activeTab.value === 'downloading') return '没有正在下载的任务'
  if (activeTab.value === 'completed') return '没有已完成的任务'
  if (activeTab.value === 'paused') return '没有已暂停的任务'
  if (activeTab.value === 'failed') return '没有失败的任务'
  return '暂无下载任务'
}

const getEmptyStateMessage = (): string => {
  if (searchQuery.value) return '尝试使用其他关键词搜索'
  if (activeTab.value === 'all') return '前往番剧页面开始下载'
  return '切换到其他标签查看任务'
}

// 操作方法
const pauseDownload = async (taskId: number) => {
  try {
    await downloadStore.pauseDownload(taskId)
    feedbackStore.showToast('已暂停下载', 'success')
  } catch (error: any) {
    feedbackStore.showError(error?.message || '暂停下载失败')
  }
}

const resumeDownload = async (taskId: number) => {
  try {
    await downloadStore.resumeDownload(taskId)
    feedbackStore.showToast('已恢复下载', 'success')
  } catch (error: any) {
    feedbackStore.showError(error?.message || '恢复下载失败')
  }
}

const deleteDownload = async (taskId: number) => {
  try {
    await downloadStore.removeDownload(taskId, true)
    feedbackStore.showToast('已删除下载任务', 'success')
  } catch (error: any) {
    feedbackStore.showError(error?.message || '删除下载任务失败')
  }
}

const retryDownload = async (taskId: number) => {
  // TODO: 实现重试逻辑
  feedbackStore.showToast('重试功能开发中', 'info')
}

const pauseAllDownloads = async () => {
  try {
    for (const task of downloadingTasks.value) {
      await downloadStore.pauseDownload(task.id)
    }
    feedbackStore.showToast('已暂停所有下载', 'success')
  } catch (error: any) {
    feedbackStore.showError('批量暂停失败')
  }
}

const resumeAllDownloads = async () => {
  try {
    for (const task of pausedTasks.value) {
      await downloadStore.resumeDownload(task.id)
    }
    feedbackStore.showToast('已恢复所有下载', 'success')
  } catch (error: any) {
    feedbackStore.showError('批量恢复失败')
  }
}

const clearCompleted = async () => {
  try {
    for (const task of completedTasks.value) {
      await downloadStore.removeDownload(task.id, true)
    }
    feedbackStore.showToast('已清空已完成任务', 'success')
  } catch (error: any) {
    feedbackStore.showError('清空失败')
  }
}

const retryAllFailed = () => {
  feedbackStore.showToast('批量重试功能开发中', 'info')
}

const openDownloadFolder = async () => {
  const download_folder = await downloadStore.openDownloadFolder()
  if (!download_folder) {
    feedbackStore.showError('未找到下载目录');
    return;
  }
  try {
    await downloadStore.openFilePath(download_folder)
    feedbackStore.showToast('已在文件管理器中打开', 'success');
  } catch (error: any) {
    feedbackStore.showError('打开文件失败');
  }
}

const openFile = async (taskId: number) => {
  const download_path = await downloadStore.getDownloadPath(taskId)
  if (!download_path) {
    feedbackStore.showError('未找到文件路径');
    return;
  }
  try {
    await downloadStore.openFilePath(download_path)
    feedbackStore.showToast('已在文件管理器中打开', 'success');
  } catch (error: any) {
    feedbackStore.showError('打开文件失败');
  }
}

const showContextMenu = (event: MouseEvent, task: any) => {
  // TODO: 实现右键菜单
  console.log('Right click on task:', task)
}

// 初始化
onMounted(async () => {
  loading.value = true
  try {
    await downloadStore.init()
    await downloadStore.fetchAllDownloads()
  } catch (error) {
    feedbackStore.showError('加载下载任务失败')
  } finally {
    loading.value = false
  }
})

// 点击外部关闭更多操作菜单
const handleClickOutside = (event: MouseEvent) => {
  if (!event.target || !(event.target as Element).closest('.action-left')) {
    showMoreActions.value = false
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})
</script>

<style scoped>
.download-management-view {
  padding: 2rem;
  max-width: 1200px;
  margin: 0 auto;
}

/* 状态标签 */
.status-tabs {
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid var(--color-border);
  margin-bottom: 1.5rem;
}

.tab-item {
  position: relative;
  padding: 1rem 0;
  margin-right: 2rem;
  cursor: pointer;
  color: var(--color-text-light);
  font-weight: 500;
  transition: color 0.3s ease;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.tab-item:hover {
  color: var(--color-primary);
}

.tab-item.active {
  color: var(--color-primary);
}

.tab-item.active::after {
  content: '';
  position: absolute;
  bottom: -1px;
  left: 0;
  right: 0;
  height: 2px;
  background: var(--color-primary);
}

.tab-count {
  background: var(--color-bg-light);
  color: var(--color-text-light);
  padding: 0.125rem 0.5rem;
  border-radius: 12px;
  font-size: 0.75rem;
  min-width: 1.5rem;
  text-align: center;
}

.tab-item.active .tab-count {
  background: var(--color-primary);
  color: white;
}

.tab-actions {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.action-link {
  color: var(--color-text-light);
  text-decoration: none;
  font-size: 0.9rem;
  padding: 0.5rem 1rem;
  border-radius: 6px;
  transition: all 0.3s ease;
  background: none;
  border: none;
  cursor: pointer;
}

.action-link:hover {
  background: var(--color-bg-light);
  color: var(--color-primary);
}

/* 操作栏 */
.action-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1rem;
  padding: 1rem;
  background: var(--color-bg-light);
  border-radius: 8px;
}

.action-left {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  position: relative;
}

.action-btn {
  padding: 0.5rem 1rem;
  border: none;
  border-radius: 6px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.3s ease;
  font-size: 0.9rem;
}

.action-btn.primary {
  background: var(--color-primary);
  color: white;
}

.action-btn.primary:hover {
  background: var(--color-primary-dark);
}

.action-btn.secondary {
  background: white;
  color: var(--color-text);
  border: 1px solid var(--color-border);
}

.action-btn.secondary:hover {
  background: var(--color-bg-light);
}

.more-actions-dropdown {
  position: absolute;
  top: 100%;
  left: 0;
  background: white;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  box-shadow: var(--shadow-md);
  z-index: 100;
  min-width: 120px;
  margin-top: 0.5rem;
}

.more-actions-dropdown button {
  display: block;
  width: 100%;
  padding: 0.75rem 1rem;
  background: none;
  border: none;
  text-align: left;
  cursor: pointer;
  transition: background 0.2s ease;
}

.more-actions-dropdown button:hover {
  background: var(--color-bg-light);
}

.action-right {
  display: flex;
  align-items: center;
}

.search-box {
  position: relative;
  display: flex;
  align-items: center;
}

.search-input {
  padding: 0.5rem 2.5rem 0.5rem 1rem;
  border: 1px solid var(--color-border);
  border-radius: 20px;
  font-size: 0.9rem;
  width: 200px;
  transition: all 0.3s ease;
}

.search-input:focus {
  width: 240px;
  border-color: var(--color-primary);
  box-shadow: 0 0 0 3px rgba(52, 152, 219, 0.1);
}

.search-icon {
  position: absolute;
  right: 1rem;
  color: var(--color-text-light);
  pointer-events: none;
}

/* 表格头部 */
.table-header {
  display: flex;
  align-items: center;
  padding: 0.75rem 1rem;
  background: var(--color-bg-light);
  border-radius: 6px 6px 0 0;
  font-weight: 600;
  color: var(--color-text-light);
  font-size: 0.85rem;
}

/* 任务列表 */
.task-list {
  background: white;
  border-radius: 0 0 8px 8px;
  overflow: hidden;
}

.task-row {
  display: flex;
  align-items: center;
  padding: 0.75rem 1rem;
  border-bottom: 1px solid var(--color-border-light);
  transition: all 0.2s ease;
  position: relative;
}

.task-row:hover {
  background: rgba(0, 0, 0, 0.02);
}

.task-row:last-child {
  border-bottom: none;
}

/* 状态样式 */
.task-row.status-downloading {
  border-left: 3px solid #3498db;
  background: rgba(52, 152, 219, 0.02);
}

.task-row.status-completed {
  border-left: 3px solid #27ae60;
  background: rgba(39, 174, 96, 0.02);
}

.task-row.status-paused {
  border-left: 3px solid #f39c12;
  background: rgba(243, 156, 18, 0.02);
}

.task-row.status-failed {
  border-left: 3px solid #e74c3c;
  background: rgba(231, 76, 60, 0.02);
}

/* 列样式 */
.col {
  display: flex;
  align-items: center;
}

.col-index {
  width: 60px;
  justify-content: center;
  font-family: 'SF Mono', 'Monaco', monospace;
  font-weight: 600;
  color: var(--color-text-light);
}

.col-title {
  flex: 1;
  min-width: 0;
}

.col-status {
  width: 200px;
}

.col-time {
  width: 100px;
  font-size: 0.85rem;
  color: var(--color-text-light);
}

.col-size {
  width: 80px;
  font-size: 0.85rem;
  color: var(--color-text-light);
  text-align: right;
}

.col-actions {
  width: 120px;
  justify-content: center;
}

/* 任务信息 */
.task-info {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  min-width: 0;
}

.task-cover {
  width: 40px;
  height: 56px;
  object-fit: cover;
  border-radius: 4px;
  flex-shrink: 0;
}

.task-details {
  min-width: 0;
  flex: 1;
}

.task-name {
  font-weight: 500;
  color: var(--color-text);
  margin-bottom: 0.25rem;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.task-episode {
  font-size: 0.85rem;
  color: var(--color-text-light);
}

/* 状态内容 */
.status-content {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.progress-bar {
  width: 120px;
  height: 4px;
  background: var(--color-border-light);
  border-radius: 2px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: var(--color-primary);
  transition: width 0.3s ease;
  position: relative;
}

.progress-fill::after {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: linear-gradient(90deg, transparent, rgba(255,255,255,0.3), transparent);
  animation: shimmer 2s infinite;
}

@keyframes shimmer {
  0% { transform: translateX(-100%); }
  100% { transform: translateX(100%); }
}

.progress-text {
  font-size: 0.75rem;
  color: var(--color-text-light);
}

.status-icon {
  margin-right: 0.25rem;
}

.retry-btn {
  margin-left: 0.5rem;
  padding: 0.125rem 0.5rem;
  background: var(--color-error);
  color: white;
  border: none;
  border-radius: 4px;
  font-size: 0.75rem;
  cursor: pointer;
}

.retry-btn:hover {
  background: #c0392b;
}

/* 操作按钮 */
.action-buttons {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.action-icon {
  width: 32px;
  height: 32px;
  border: none;
  background: none;
  cursor: pointer;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
  font-size: 0.9rem;
}

.action-icon:hover {
  background: var(--color-bg-light);
}

.action-icon.delete:hover {
  background: rgba(231, 76, 60, 0.1);
}

/* 空状态 */
.empty-state {
  text-align: center;
  padding: 4rem 2rem;
}

.empty-icon {
  font-size: 4rem;
  margin-bottom: 1rem;
}

.empty-state h3 {
  margin-bottom: 0.5rem;
  color: var(--color-text);
}

.empty-state p {
  color: var(--color-text-light);
  margin-bottom: 2rem;
}

/* 加载状态 */
.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 4rem 2rem;
  gap: 1rem;
}

.loading-spinner {
  width: 40px;
  height: 40px;
  border: 3px solid var(--color-border-light);
  border-top-color: var(--color-primary);
  border-radius: 50%;
  animation: spin 1s infinite linear;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* 响应式设计 */
@media (max-width: 1024px) {
  .col-time {
    display: none;
  }
  
  .col-size {
    width: 60px;
  }
}

@media (max-width: 768px) {
  .download-management-view {
    padding: 1rem;
  }
  
  .action-bar {
    flex-direction: column;
    gap: 1rem;
  }
  
  .action-left,
  .action-right {
    width: 100%;
    justify-content: center;
  }
  
  .search-input {
    width: 100%;
  }
  
  .search-input:focus {
    width: 100%;
  }
  
  .col-index {
    width: 40px;
  }
  
  .col-size {
    display: none;
  }
  
  .col-actions {
    width: 80px;
  }
  
  .task-cover {
    width: 32px;
    height: 45px;
  }
  
  .task-name {
    font-size: 0.9rem;
  }
  
  .task-episode {
    font-size: 0.8rem;
  }
}

@media (max-width: 480px) {
  .status-tabs {
    flex-wrap: wrap;
    gap: 0.5rem;
  }
  
  .tab-item {
    margin-right: 1rem;
    padding: 0.75rem 0;
  }
  
  .tab-actions {
    width: 100%;
    justify-content: center;
    margin-top: 0.5rem;
  }
  
  .col-status {
    width: 140px;
  }
  
  .progress-bar {
    width: 80px;
  }
}
</style> 