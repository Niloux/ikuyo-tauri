<template>
  <div class="anime-resources">
    <!-- 筛选控制栏 -->
    <div class="filters-bar">
      <div class="filters-left">
        <h3>番剧资源</h3>
        <span v-if="resourcesData" class="total-count">
          共 {{ resourcesData.total_resources }} 个资源
        </span>
      </div>

      <div class="filters-right">
        <select v-model="selectedResolution" @change="handleFilterChange" class="filter-select">
          <option value="">全部分辨率</option>
          <option value="1080p">1080p</option>
          <option value="720p">720p</option>
          <option value="4K">4K</option>
        </select>

        <select v-model="selectedSubtitleType" @change="handleFilterChange" class="filter-select">
          <option value="">全部字幕</option>
          <option value="简体中文">简体中文</option>
          <option value="繁体中文">繁体中文</option>
          <option value="中日双语">中日双语</option>
          <option value="简繁双语">简繁双语</option>
          <option value="无字幕">无字幕</option>
        </select>

        <button @click="refreshResources" class="refresh-btn" :disabled="loading">
          {{ loading ? '刷新中...' : '刷新' }}
        </button>
      </div>
    </div>

    <!-- 加载状态 -->
    <div v-if="loading" class="loading-state">
      <div class="loading-spinner"></div>
      <p>正在加载全部资源...</p>
    </div>

    <!-- 错误状态 -->
    <div v-else-if="error" class="error-state">
      <div class="error-icon">⚠️</div>
      <p>{{ error }}</p>
      <button @click="refreshResources" class="retry-btn">重试</button>
    </div>

    <!-- 资源列表 -->
    <div v-else-if="resourcesData && resourcesData.subtitle_groups.length > 0" class="resources-content">
      <!-- 按字幕组分类的资源列表 -->
      <div class="subtitle-groups">
        <div
          v-for="group in resourcesData.subtitle_groups"
          :key="group.id"
          class="subtitle-group"
        >
          <div
            class="group-header"
            :class="{ 'expanded': isGroupExpanded(group.id) }"
            @click="toggleGroup(group.id)"
          >
            <div class="group-info">
              <h4 class="group-name">{{ group.name }}</h4>
              <span class="group-count">{{ group.resource_count }} 个资源</span>
            </div>
            <div class="expand-icon" :class="{ 'expanded': isGroupExpanded(group.id) }">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="m9 18 6-6-6-6"/>
              </svg>
            </div>
          </div>

          <transition name="expand-collapse">
            <div v-show="isGroupExpanded(group.id)" class="group-resources">
              <div
                v-for="resource in group.resources"
                :key="resource.id"
                class="resource-item"
              >
                <div class="resource-info">
                  <div class="resource-title">{{ resource.title }}</div>
                  <div class="resource-meta">
                    <span v-if="resource.resolution" class="meta-tag resolution">
                      {{ resource.resolution }}
                    </span>
                    <span v-if="resource.subtitle_type" class="meta-tag subtitle">
                      {{ resource.subtitle_type }}
                    </span>
                    <span v-if="resource.size" class="meta-tag size">
                      {{ resource.size }}
                    </span>
                    <span v-if="resource.release_date" class="meta-tag date">
                      {{ formatReleaseDate(resource.release_date) }}
                    </span>
                  </div>
                </div>

                <div class="resource-actions">
                  <button
                    v-if="resource.magnet_url"
                    @click="downloadMagnet(resource.magnet_url)"
                    class="action-btn magnet-btn"
                    title="磁力链接"
                  >
                    磁力
                  </button>
                  <button
                    v-if="resource.torrent_url"
                    @click="downloadTorrent(resource.torrent_url)"
                    class="action-btn torrent-btn"
                    title="种子下载"
                  >
                    种子
                  </button>
                </div>
              </div>
            </div>
          </transition>
        </div>
      </div>

      <!-- 分页控制（如果需要） -->
      <div v-if="needsPagination" class="pagination-controls">
        <button
          @click="loadPreviousPage"
          :disabled="!hasPreviousPage || loading"
          class="pagination-btn"
        >
          上一页
        </button>

        <span class="pagination-info">
          显示 {{ currentOffset + 1 }}-{{ Math.min(currentOffset + currentLimit, totalResources) }}
          / 共 {{ totalResources }} 个
        </span>

        <button
          @click="loadNextPage"
          :disabled="!hasNextPage || loading"
          class="pagination-btn"
        >
          下一页
        </button>
      </div>
    </div>

    <!-- 空状态 -->
    <div v-else class="empty-state">
      <div class="empty-icon">📦</div>
      <p>该番剧暂无可用资源</p>
      <p class="empty-subtitle">可能还没有字幕组发布资源，请稍后再试</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { useResourceStore } from '../stores/resourceStore'
import { useFeedbackStore } from '../stores/feedbackStore'

// Props定义
interface Props {
  bangumiId: number
}
const props = defineProps<Props>()

const resourceStore = useResourceStore()
const feedbackStore = useFeedbackStore()

// 分页和筛选状态
const selectedResolution = ref('')
const selectedSubtitleType = ref('')
const currentLimit = ref(100)
const currentOffset = ref(0)
const fullResources = ref(9999)

// 组装查询参数
const getQuery = () => ({
  bangumiId: props.bangumiId,
  resolution: selectedResolution.value || undefined,
  subtitleType: selectedSubtitleType.value || undefined,
  limit: fullResources.value,
  offset: 0
})

// 监听筛选和分页变化自动拉取数据
watch([
  () => props.bangumiId,
  selectedResolution,
  selectedSubtitleType,
  currentLimit,
  currentOffset
], () => {
  if (props.bangumiId) {
    resourceStore.fetchResources(getQuery())
  }
}, { immediate: true })

// 计算属性
const resourcesData = computed(() => resourceStore.resourcesData)
const loading = computed(() => resourceStore.loading)
const error = computed(() => resourceStore.error)

const totalResources = computed(() => resourcesData.value?.total_resources || 0)
const needsPagination = computed(() => totalResources.value > currentLimit.value)
const hasPreviousPage = computed(() => currentOffset.value > 0)
const hasNextPage = computed(() =>
  currentOffset.value + currentLimit.value < totalResources.value
)

// 优化：缓存日期格式化选项，避免重复创建
const dateFormatOptions: Intl.DateTimeFormatOptions = {
  month: 'short',
  day: 'numeric',
  hour: 'numeric',
  minute: '2-digit'
}

// 优化：格式化发布日期
const formatReleaseDate = (dateStr: string): string => {
  if (!dateStr) return ''
  try {
    const date = new Date(Number(dateStr))
    return date.toLocaleDateString('zh-CN', dateFormatOptions)
  } catch {
    return dateStr
  }
}

// 处理筛选变化
const handleFilterChange = () => {
  currentOffset.value = 0 // 重置到第一页
  resourceStore.fetchResources(getQuery())
}

// 刷新资源
const refreshResources = () => {
  resourceStore.fetchResources(getQuery())
}

// 分页控制
const loadPreviousPage = () => {
  if (hasPreviousPage.value) {
    currentOffset.value = Math.max(0, currentOffset.value - currentLimit.value)
  }
}
const loadNextPage = () => {
  if (hasNextPage.value) {
    currentOffset.value += currentLimit.value
  }
}

// 折叠状态管理
const expandedGroups = ref<Set<number>>(new Set())
const toggleGroup = (groupId: number) => {
  const newExpandedGroups = new Set(expandedGroups.value)
  if (newExpandedGroups.has(groupId)) {
    newExpandedGroups.delete(groupId)
  } else {
    newExpandedGroups.add(groupId)
  }
  expandedGroups.value = newExpandedGroups
}
const isGroupExpanded = (groupId: number): boolean => {
  return expandedGroups.value.has(groupId)
}

// 磁力链接下载逻辑
const downloadMagnet = async (url: string) => {
  if (!url) return
  try {
    const { openUrl } = await import('@tauri-apps/plugin-opener')
    await openUrl(url)
  } catch (err) {
    // openUrl 失败，自动复制磁力链接到剪贴板
    let copied = false
    try {
      const { writeText } = await import('@tauri-apps/plugin-clipboard-manager')
      await writeText(url)
      copied = true
    } catch (e1) {
      try {
        await navigator.clipboard.writeText(url)
        copied = true
      } catch (e2) {
        copied = false
      }
    }
    if (copied) {
      feedbackStore.showError('未检测到下载工具，磁力链接已复制，请手动粘贴到下载器')
    } else {
      feedbackStore.showError('磁力链接复制失败，请手动复制')
    }
  }
}

// 种子文件下载逻辑
const downloadTorrent = async (url: string) => {
  if (!url) return
  try {
    const link = document.createElement('a')
    link.href = url
    link.download = '' // 让浏览器/tauri决定文件名
    link.target = '_blank'
    document.body.appendChild(link)
    link.click()
    document.body.removeChild(link)
  } catch (err) {
    feedbackStore.showError('下载失败，请检查链接或重试')
  }
}
</script>

<style scoped>
.anime-resources {
  padding: 1.5rem 0;
}

/* 筛选控制栏 */
.filters-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 2rem;
  padding: 1rem 1.5rem;
  background: white;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.filters-left h3 {
  margin: 0 1rem 0 0;
  color: #2c3e50;
  font-size: 1.25rem;
}

.total-count {
  color: #7f8c8d;
  font-size: 0.9rem;
}

/* 新统一风格样式，参考.sort-select.unified-input */
.filter-select {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
  font-weight: 400;
  color: #333;
  appearance: none;
  -webkit-appearance: none;
  -moz-appearance: none;
  background: #fff;
  border: 1px solid #d1d5db;
  border-radius: 12px;
  font-size: 16px;
  height: 44px;
  padding: 0 40px 0 16px;
  box-shadow: 0 1px 3px rgba(0,0,0,0.06);
  transition: border-color 0.2s, box-shadow 0.2s, background 0.2s;
  background-image: url('data:image/svg+xml;utf8,<svg fill="%23666" height="20" viewBox="0 0 20 20" width="20" xmlns="http://www.w3.org/2000/svg"><path d="M5.8 8.3a1 1 0 0 1 1.4 0L10 11.09l2.8-2.8a1 1 0 1 1 1.4 1.42l-3.5 3.5a1 1 0 0 1-1.4 0l-3.5-3.5a1 1 0 0 1 0-1.42z"/></svg>');
  background-repeat: no-repeat;
  background-position: right 14px center;
  background-size: 20px 20px;
  outline: none;
  margin: 0;
}
.filter-select:focus {
  border-color: #3498db;
  box-shadow: 0 0 0 2px rgba(52,152,219,0.15);
}

/* 统一刷新按钮风格 */
.refresh-btn {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
  font-weight: 400;
  color: #fff;
  background: #3498db;
  border: 1px solid #3498db;
  border-radius: 12px;
  font-size: 16px;
  height: 44px;
  padding: 0 24px;
  box-shadow: 0 1px 3px rgba(0,0,0,0.06);
  transition: background 0.2s, border-color 0.2s, box-shadow 0.2s;
  cursor: pointer;
  outline: none;
  margin: 0;
  display: inline-block;
}
.refresh-btn:hover:not(:disabled) {
  background: #217dbb;
  border-color: #217dbb;
}
.refresh-btn:active {
  background: #17609c;
  border-color: #17609c;
}
.refresh-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

/* 筛选栏排版优化 */
.filters-right {
  display: flex;
  gap: 1rem;
  align-items: center;
}

/* 保证控件高度一致 */
.filters-right .filter-select,
.filters-right .refresh-btn {
  height: 44px;
  min-width: 120px;
}

/* 状态样式 */
.loading-state, .error-state, .empty-state {
  text-align: center;
  padding: 3rem;
  color: #7f8c8d;
}

.loading-spinner {
  width: 40px;
  height: 40px;
  border: 4px solid #f3f3f3;
  border-top: 4px solid #3498db;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin: 0 auto 1rem;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.error-icon, .empty-icon {
  font-size: 3rem;
  margin-bottom: 1rem;
}

.retry-btn {
  background: #e74c3c;
  color: white;
  border: none;
  padding: 0.75rem 1.5rem;
  border-radius: 6px;
  cursor: pointer;
  font-weight: 500;
  transition: background-color 0.3s;
  margin-top: 1rem;
}

.retry-btn:hover {
  background: #c0392b;
}

.empty-subtitle {
  color: #bdc3c7;
  font-size: 0.9rem;
  margin-top: 0.5rem;
}

/* 资源列表样式 */
.resources-content {
  background: white;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  overflow: hidden;
}

.subtitle-groups {
  /* space-y: 0 - 使用 margin 代替 */
}

.subtitle-group {
  border-bottom: 1px solid #f0f0f0;
}

.subtitle-group:last-child {
  border-bottom: none;
}

.group-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1rem 1.5rem;
  background-color: #f8f9fa;
  border-bottom: 1px solid #e0e0e0;
  cursor: pointer;
  transition: all 0.3s ease;
  user-select: none;
}

.group-header:hover {
  background-color: #e9ecef;
  transform: translateY(-1px);
}

.group-header.expanded {
  background-color: #e3f2fd;
  border-bottom-color: #2196f3;
}

.group-info {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.group-name {
  font-size: 1.1rem;
  font-weight: 600;
  color: #2c3e50;
  margin: 0;
}

.group-count {
  font-size: 0.85rem;
  color: #7f8c8d;
  background-color: #e9ecef;
  padding: 0.25rem 0.5rem;
  border-radius: 12px;
}

.expand-icon {
  width: 24px;
  height: 24px;
  color: #6c757d;
  transition: transform 0.3s ease;
  display: flex;
  align-items: center;
  justify-content: center;
}

.expand-icon.expanded {
  transform: rotate(90deg);
  color: #2196f3;
}

.expand-icon svg {
  width: 16px;
  height: 16px;
}

.group-resources {
  padding: 0.5rem 0;
  overflow: hidden;
}

/* 展开收起动画 */
.expand-collapse-enter-active,
.expand-collapse-leave-active {
  transition: all 0.4s ease;
  transform-origin: top;
}

.expand-collapse-enter-from,
.expand-collapse-leave-to {
  opacity: 0;
  transform: scaleY(0);
  max-height: 0;
}

.expand-collapse-enter-to,
.expand-collapse-leave-from {
  opacity: 1;
  transform: scaleY(1);
  max-height: 2000px;
}

.resource-item {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  padding: 1rem 1.5rem;
  border-bottom: 1px solid #f8f9fa;
  transition: all 0.2s ease;
}

.resource-item:hover {
  background-color: #f8f9fa;
}

.resource-item:last-child {
  border-bottom: none;
}

.resource-info {
  flex: 1;
  margin-right: 1rem;
}

.resource-title {
  font-weight: 500;
  color: #2c3e50;
  margin-bottom: 0.5rem;
  line-height: 1.4;
  font-size: 0.95rem;
}

.resource-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
}

.meta-tag {
  font-size: 0.75rem;
  padding: 0.25rem 0.5rem;
  border-radius: 12px;
  font-weight: 500;
}

.meta-tag.resolution {
  background-color: #e3f2fd;
  color: #1976d2;
}

.meta-tag.subtitle {
  background-color: #f3e5f5;
  color: #7b1fa2;
}

.meta-tag.size {
  background-color: #e8f5e8;
  color: #388e3c;
}

.meta-tag.date {
  background-color: #fff3e0;
  color: #f57c00;
}

.resource-actions {
  display: flex;
  gap: 0.5rem;
}

.magnet-btn {
  background-color: #e74c3c;
  color: white;
  border: none;
  border-radius: 6px;
  padding: 0.5rem 1rem;
  cursor: pointer;
  font-weight: 500;
  font-size: 0.85rem;
  min-width: 60px;
  transition: all 0.3s;
}
.magnet-btn:hover {
  background-color: #c0392b;
  transform: translateY(-1px);
}

.torrent-btn {
  background-color: #3498db;
  color: white;
  border: none;
  border-radius: 6px;
  padding: 0.5rem 1rem;
  cursor: pointer;
  font-weight: 500;
  font-size: 0.85rem;
  min-width: 60px;
  transition: all 0.3s;
}
.torrent-btn:hover {
  background-color: #2980b9;
  transform: translateY(-1px);
}

.download-btn {
  padding: 0.5rem 1rem;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-weight: 500;
  font-size: 0.85rem;
  transition: all 0.3s;
  min-width: 60px;
}

/* 分页控制 */
.pagination-controls {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1rem 1.5rem;
  border-top: 1px solid #f0f0f0;
  background-color: #fafafa;
}

.pagination-btn {
  background: #3498db;
  color: white;
  border: none;
  padding: 0.5rem 1rem;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.9rem;
  transition: background-color 0.3s;
}

.pagination-btn:hover:not(:disabled) {
  background: #2980b9;
}

.pagination-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.pagination-info {
  font-size: 0.9rem;
  color: #7f8c8d;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .filters-bar {
    flex-direction: column;
    gap: 1rem;
    align-items: stretch;
  }
  .filters-right {
    flex-wrap: wrap;
    justify-content: center;
    gap: 0.75rem;
  }
  .filters-right .filter-select,
  .filters-right .refresh-btn {
    width: 100%;
    min-width: 0;
  }

  .resource-item {
    flex-direction: column;
    gap: 1rem;
  }

  .resource-actions {
    align-self: flex-start;
  }

  .pagination-controls {
    flex-direction: column;
    gap: 1rem;
  }
}

@media (max-width: 480px) {
  .anime-resources {
    padding: 1rem 0;
  }

  .filters-bar {
    padding: 1rem;
  }

  .resource-item {
    padding: 1rem;
  }

  .group-header {
    padding: 0.75rem 1rem;
  }

  .filter-select {
    width: 100%;
    margin-bottom: 0.5rem;
  }

  .filters-right {
    width: 100%;
  }
}
</style>
