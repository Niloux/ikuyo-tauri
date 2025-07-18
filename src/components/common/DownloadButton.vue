<template>
  <div class="download-button-container" ref="containerRef">
    <button
      class="download-btn"
      :class="[statusClass, { disabled }]"
      :disabled="disabled"
      @click="handleClick"
      :style="{ '--progress-width': progressPercent + '%' }"
      :aria-label="buttonText"
      :title="tooltipText"
    >
      {{ buttonText }}
      <svg 
        v-if="uiState.status === 'downloading'" 
        class="dropdown-arrow" 
        viewBox="0 0 24 24"
      >
        <path d="M7 10l5 5 5-5z" fill="currentColor"/>
      </svg>
    </button>
    
    <!-- 下拉菜单 -->
    <div 
      v-if="showMenu && uiState.status === 'downloading'" 
      class="download-menu"
      @click.stop
    >
      <div class="menu-item" @click="handleAction('pause')">
        暂停
      </div>
      <div class="menu-item danger" @click="handleAction('delete')">
        删除
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from 'vue'
import { useDownloadStore } from '@/stores/downloadStore'

const props = defineProps<{
  resourceId: number
  onAction?: (action: 'download' | 'pause' | 'resume' | 'delete' | 'retry') => void
}>()

const downloadStore = useDownloadStore()
const uiState = computed(() => downloadStore.getTaskUIState(props.resourceId))

const showMenu = ref(false)
const containerRef = ref<HTMLElement>()

const progressPercent = computed(() => Math.max(0, Math.min(100, Math.round((uiState.value.progress || 0) * 100))))
const statusClass = computed(() => {
  switch (uiState.value.status) {
    case 'downloading': return 'downloading'
    case 'completed': return 'completed'
    case 'failed': return 'failed'
    case 'paused': return 'paused'
    case 'pending': return 'pending'
    default: return ''
  }
})
const buttonText = computed(() => uiState.value.buttonText)
const disabled = computed(() => uiState.value.disabled)
const tooltipText = computed(() => {
  if (uiState.value.errorMsg) return uiState.value.errorMsg
  if (uiState.value.status === 'downloading') {
    let tip = `进度: ${progressPercent.value}%`
    if (uiState.value.speed) tip += `\n速度: ${uiState.value.speed.toFixed(2)} MB/s`
    if (uiState.value.timeRemaining) tip += `\n剩余: ${uiState.value.timeRemaining}`
    return tip
  }
  return ''
})

const handleClick = (e: Event) => {
  if (disabled.value) return
  if (uiState.value.status === 'downloading') {
    toggleMenu()
    return
  }
  let action: 'download' | 'pause' | 'resume' | 'delete' | 'retry'
  switch (uiState.value.status) {
    case 'completed': action = 'delete'; break
    case 'failed': action = 'retry'; break
    case 'paused': action = 'resume'; break
    default: action = 'download'; break
  }
  props.onAction && props.onAction(action)
}

const toggleMenu = () => {
  showMenu.value = !showMenu.value
}

const handleAction = (action: 'download' | 'pause' | 'resume' | 'delete' | 'retry') => {
  showMenu.value = false
  props.onAction && props.onAction(action)
}

const handleClickOutside = (event: Event) => {
  if (containerRef.value && !containerRef.value.contains(event.target as Node)) {
    showMenu.value = false
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
.download-button-container {
  position: relative;
  display: inline-block;
}

.download-btn {
  /* meta-tag 基础样式 */
  font-size: 0.75rem;
  padding: 0.25rem 0.5rem;
  border-radius: 12px;
  font-weight: 500;
  
  /* 固定宽度 */
  width: 80px;
  min-width: 80px;
  
  /* 基础样式 */
  border: none;
  cursor: pointer;
  position: relative;
  overflow: hidden;
  transition: all 0.2s ease;
  
  /* 默认状态 - 类似 meta-tag.resolution */
  background-color: #e3f2fd;
  color: #1976d2;
  
  /* 基础布局 */
  display: flex;
  align-items: center;
  justify-content: center;
}

/* 下载中状态调整文字位置和布局 */
.download-btn.downloading {
  justify-content: center; /* 文字居中 */
  position: relative; /* 为绝对定位的箭头提供参考 */
}

/* 非下载中状态文字居中 */
.download-btn:not(.downloading) {
  justify-content: center;
}

/* 下拉箭头样式 */
.dropdown-arrow {
  position: absolute;
  right: 8px;
  top: 50%;
  transform: translateY(-50%);
  width: 16px;
  height: 16px;
  cursor: pointer;
  transition: transform 0.2s ease;
}

.dropdown-arrow:hover {
  transform: scale(1.1);
}

/* 内嵌进度条 */
.download-btn::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: var(--progress-width, 0%);
  background: rgba(255, 255, 255, 0.4);
  transition: width 0.3s ease;
  z-index: 1;
  border-radius: 12px 0 0 12px;
}

.download-btn > * {
  position: relative;
  z-index: 2;
}

/* 确保文字在进度条上方 */
.download-btn {
  z-index: 0;
}

/* 状态样式 */
.download-btn.downloading {
  background-color: #e3f2fd;
  color: #1976d2;
  cursor: pointer;
  justify-content: space-between;
}

/* 下载中状态的进度条更明显 */
.download-btn.downloading::before {
  background: rgba(25, 118, 210, 0.3);
}

.download-btn.completed {
  background-color: #e8f5e8;
  color: #388e3c;
}

.download-btn.failed {
  background-color: #ffebee;
  color: #e74c3c;
}

.download-btn.paused {
  background-color: #fffde7;
  color: #f1c40f;
}

.download-btn.pending {
  background-color: #f0f0f0;
  color: #888;
}

.download-btn.disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

/* 悬停效果 */
.download-btn:not(.disabled):hover {
  transform: translateY(-1px);
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

/* 下拉菜单样式 */
.download-menu {
  position: absolute;
  top: 100%;
  right: 0;
  margin-top: 4px;
  background: white;
  border: 1px solid #e0e0e0;
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  z-index: 1000;
  min-width: 80px;
  animation: menuFadeIn 0.2s ease;
}

@keyframes menuFadeIn {
  from {
    opacity: 0;
    transform: translateY(-8px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.menu-item {
  padding: 8px 12px;
  cursor: pointer;
  font-size: 0.75rem;
  color: #2c3e50;
  transition: background-color 0.2s;
  border-radius: 4px;
  margin: 2px;
}

.menu-item:hover {
  background-color: #f8f9fa;
}

.menu-item.danger {
  color: #e74c3c;
}

.menu-item.danger:hover {
  background-color: #ffebee;
}
</style> 