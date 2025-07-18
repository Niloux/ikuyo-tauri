<template>
  <div class="download-button-container" ref="containerRef">
    <button
      class="download-btn"
      :class="[statusClass, { disabled }]"
      :disabled="disabled"
      @click="handleClick"
      :style="{ '--progress-width': progressPercent + '%', '--progress-radius': progressPercent === 100 ? '12px' : '12px 0 0 12px' }"
      :aria-label="buttonText"
      :title="tooltipText"
    >
      {{ buttonText }}
    </button>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useDownloadStore } from '@/stores/downloadStore'

const props = defineProps<{
  resourceId: number
  onAction?: (action: 'download' | 'pause' | 'resume' | 'delete' | 'retry') => void
}>()

const downloadStore = useDownloadStore()
const uiState = computed(() => downloadStore.getTaskUIState(props.resourceId))

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
    props.onAction && props.onAction('pause')
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
</script>

<style scoped>
.download-button-container {
  position: relative;
  display: inline-block;
}

.download-btn {
  font-size: 0.75rem;
  padding: 0.25rem 0.5rem;
  border-radius: 12px;
  font-weight: 500;
  width: 70px;
  min-width: 70px;
  border: none;
  cursor: pointer;
  position: relative;
  overflow: hidden;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 0;
}

.download-btn::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: var(--progress-width, 0%);
  border-radius: var(--progress-radius, 12px 0 0 12px);
  z-index: 1;
  transition: width 0.3s cubic-bezier(.4,0,.2,1), background 0.2s, border-radius 0.2s;
  opacity: 1;
}

.download-btn > * {
  position: relative;
  z-index: 2;
}

/* downloading - 主色蓝 */
.download-btn.downloading {
  background-color: #e3f2fd;
  color: #1976d2;
}
.download-btn.downloading::before {
  background: linear-gradient(90deg, rgba(25,118,210,0.2) 0%, rgba(25,118,210,0.4) 100%);
  opacity: 1;
}

/* completed - 绿色 */
.download-btn.completed {
  background-color: #43a047;
  color: #fff;
}
.download-btn.completed::before {
  background: linear-gradient(90deg, rgba(255,255,255,0.15) 0%, rgba(255,255,255,0.25) 100%);
  opacity: 1;
}

/* failed - 红色 */
.download-btn.failed {
  background-color: #e74c3c;
  color: #fff;
}
.download-btn.failed::before {
  background: linear-gradient(90deg, rgba(255,255,255,0.15) 0%, rgba(255,255,255,0.25) 100%);
  opacity: 1;
}

/* paused - 橙色 */
.download-btn.paused {
  background-color: #fff3cd;
  color: #b26a00;
}
.download-btn.paused::before {
  background: linear-gradient(90deg, rgba(178,106,0,0.15) 0%, rgba(178,106,0,0.25) 100%);
  opacity: 1;
}

/* pending - 灰色 */
.download-btn.pending {
  background-color: #f0f0f0;
  color: #888;
}
.download-btn.pending::before {
  background: linear-gradient(90deg, rgba(189,189,189,0.15) 0%, rgba(189,189,189,0.25) 100%);
  opacity: 1;
}

/* disabled - 透明度降低，进度条隐藏 */
.download-btn.disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
.download-btn.disabled::before {
  opacity: 0;
}

/* 其他状态进度条不显示 */
.download-btn:not(.downloading):not(.completed):not(.failed):not(.paused):not(.pending)::before {
  opacity: 0;
}

.download-btn:not(.disabled):hover {
  transform: translateY(-1px);
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}
</style> 