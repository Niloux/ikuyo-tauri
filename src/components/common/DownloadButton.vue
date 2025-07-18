<template>
  <button
    class="download-btn"
    :class="[statusClass, { disabled }]"
    :disabled="disabled || status === 'downloading'"
    @click="handleClick"
    :style="{ '--progress-width': progressPercent + '%' }"
    :aria-label="buttonText"
    :title="tooltipText"
  >
    {{ buttonText }}
  </button>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  status: string | null
  progress: number
  disabled?: boolean
  errorMsg?: string | null
  onClick?: () => void
  speed?: number
  timeRemaining?: string
  buttonText?: string
}>()

const progressPercent = computed(() => Math.max(0, Math.min(100, Math.round((props.progress || 0) * 100))))
const statusClass = computed(() => {
  switch (props.status) {
    case 'downloading': return 'downloading'
    case 'completed': return 'completed'
    case 'failed': return 'failed'
    case 'paused': return 'paused'
    case 'pending': return 'pending'
    default: return ''
  }
})
const buttonText = computed(() => {
  switch (props.status) {
    case 'downloading': return '下载中'
    case 'completed': return '已下载'
    case 'failed': return '重试'
    case 'paused': return '已暂停'
    case 'pending': return '等待中'
    default: return '下载'
  }
})

const tooltipText = computed(() => {
  if (props.errorMsg) return props.errorMsg
  if (props.status === 'downloading') {
    let tip = `进度: ${progressPercent.value}%`
    if (props.speed) tip += `\n速度: ${props.speed.toFixed(2)} MB/s`
    if (props.timeRemaining) tip += `\n剩余: ${props.timeRemaining}`
    return tip
  }
  return ''
})
const handleClick = (e: Event) => {
  if (props.disabled || props.status === 'downloading') return
  props.onClick && props.onClick()
}
</script>

<style scoped>
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
  cursor: not-allowed;
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
.download-btn:not(.disabled):not(.downloading):hover {
  transform: translateY(-1px);
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}
</style> 