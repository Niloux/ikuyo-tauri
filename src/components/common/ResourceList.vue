<template>
  <div class="resources-list">
    <div v-if="resourcesData && resourcesData.subtitle_groups.length > 0" class="resources-content">
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
                  <DownloadButton
                    :resource-id="resource.id"
                    :on-action="(action) => handleDownloadAction(action, resource)"
                  />
                </div>
              </div>
            </div>
          </transition>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import { useFeedbackStore } from '@/stores/feedbackStore'
import { useDownloadStore } from '@/stores/downloadStore'
import { storeToRefs } from 'pinia'
import type { EpisodeResource, EpisodeResourcesData } from '@/services/bangumi/bangumiTypes'
import DownloadButton from './DownloadButton.vue'

const props = defineProps<{
  resourcesData: EpisodeResourcesData | null
  bangumiId: number
  subject: {
    name: string
    name_cn: string
    images: {
      large: string
    }
  }
}>()

const feedbackStore = useFeedbackStore()
const downloadStore = useDownloadStore()
const { tasks } = storeToRefs(downloadStore)

// Format release date
const dateFormatOptions: Intl.DateTimeFormatOptions = {
  month: 'short',
  day: 'numeric',
  hour: 'numeric',
  minute: '2-digit'
}
const formatReleaseDate = (dateStr: string): string => {
  if (!dateStr) return ''
  try {
    const date = new Date(Number(dateStr))
    return date.toLocaleDateString('zh-CN', dateFormatOptions)
  } catch {
    return dateStr
  }
}

// 展开状态管理
const expandedGroups = ref<Set<number>>(new Set());
const expandedNumber = ref(13)

// 当资源数据变化时，初始化展开状态
watch(
  () => props.resourcesData,
  (newData) => {
    const newSet = new Set<number>()
    if (newData && Array.isArray(newData.subtitle_groups)) {
      for (const group of newData.subtitle_groups) {
        if (Array.isArray(group.resources) && group.resources.length < expandedNumber.value) {
          newSet.add(group.id)
        }
      }
    }
    expandedGroups.value = newSet
  },
  { immediate: true }
)

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

const isResourceDownloading = (resourceId: number) => {
  return !!tasks.value[resourceId]
}

const getTaskByResourceId = downloadStore.getTaskByResourceId

// Download logic
const downloadMagnet = async (url: string) => {
  if (!url) return
  try {
    const { openUrl } = await import('@tauri-apps/plugin-opener')
    await openUrl(url)
  } catch (err) {
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

const downloadTorrent = async (url: string) => {
  if (!url) return
  try {
    const link = document.createElement('a')
    link.href = url
    link.download = ''
    link.target = '_blank'
    document.body.appendChild(link)
    link.click()
    document.body.removeChild(link)
  } catch (err) {
    feedbackStore.showError('下载失败，请检查链接或重试')
  }
}

const handleDownloadAction = async (action: 'download' | 'pause' | 'resume' | 'delete' | 'retry', resource: EpisodeResource) => {
  const task = getTaskByResourceId(resource.id)
  
  switch (action) {
    case 'download':
      await handleDownload(resource)
      break
    case 'pause':
      if (task?.id != undefined) {
        try {
          await downloadStore.pauseDownload(task.id)
          feedbackStore.showToast('已暂停下载', 'success')
        } catch (e: any) {
          feedbackStore.showError(e?.message || '暂停下载失败')
        }
      }
      break
    case 'resume':
      if (task?.id != undefined) {
        try {
          await downloadStore.resumeDownload(task.id)
          feedbackStore.showToast('已恢复下载', 'success')
        } catch (e: any) {
          feedbackStore.showError(e?.message || '恢复下载失败')
        }
      }
      break
    case 'delete':
      if (task?.id != undefined) {
        try {
          await downloadStore.removeDownload(task.id, true) // TODO: delete_files参数需要开发
          feedbackStore.showToast('已删除下载任务', 'success')
        } catch (e: any) {
          feedbackStore.showError(e?.message || '删除下载任务失败')
        }
      }
      break
    case 'retry':
      if (task?.id != undefined) {
        try {
          await downloadStore.removeDownload(task.id, true) // 先删除旧任务
          await handleDownload(resource) // 重新开始下载
        } catch (e: any) {
          feedbackStore.showError(e?.message || '重试下载失败')
        }
      }
      break
  }
}

const handleDownload = async (resource: EpisodeResource) => {
  // 检查必要参数
  if (!resource.magnet_url) {
    feedbackStore.showError('缺少磁力链接，无法下载')
    return
  }
  if (!props.bangumiId || !props.subject) {
    feedbackStore.showError('番剧信息缺失，无法下载')
    return
  }
  // 组装 StartDownloadTask
  const task = {
    magnet_url: resource.magnet_url,
    // save_path: '', // 可后续扩展为用户选择
    title: resource.title,
    bangumi_id: props.bangumiId,
    resource_id: resource.id,
    episode_number: resource.episode_number,
    name: props.subject.name,
    name_cn: props.subject.name_cn,
    cover: props.subject.images?.large,
    total_size: typeof resource.size === 'number' ? resource.size : 0,
  }
  try {
    await downloadStore.startDownload(task)
    feedbackStore.showToast('已添加到下载任务', 'success')
  } catch (e: any) {
    feedbackStore.showError(e?.message || '添加下载任务失败')
  }
}

onMounted(() => {
  downloadStore.fetchAllDownloads()
})

</script>

<style scoped>
.resources-content {
  background: white;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  overflow: hidden;
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
  max-height: 2000px; /* A large enough value */
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

.meta-tag.resolution { background-color: #e3f2fd; color: #1976d2; }
.meta-tag.subtitle { background-color: #f3e5f5; color: #7b1fa2; }
.meta-tag.size { background-color: #e8f5e8; color: #388e3c; }
.meta-tag.date { background-color: #fff3e0; color: #f57c00; }

.resource-actions {
  display: flex;
  gap: 0.5rem;
}

.action-btn {
    border: none;
    border-radius: 6px;
    padding: 0.5rem 1rem;
    cursor: pointer;
    font-weight: 500;
    font-size: 0.85rem;
    min-width: 60px;
    transition: all 0.3s;
}

.magnet-btn {
  background-color: #e74c3c;
  color: white;
}
.magnet-btn:hover { background-color: #c0392b; transform: translateY(-1px); }

.torrent-btn {
  background-color: #3498db;
  color: white;
}
.torrent-btn:hover { background-color: #2980b9; transform: translateY(-1px); }

</style>
