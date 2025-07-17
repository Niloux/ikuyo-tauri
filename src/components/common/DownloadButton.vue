<template>
  <button
    class="download-btn"
    :class="[statusClass, { disabled }]"
    :disabled="disabled || status === 'downloading'"
    @click="handleClick"
    :aria-label="buttonText"
    :title="tooltipText"
  >
    <div class="progress-bar-bg">
      <div
        class="progress-bar-fill"
        :style="{ width: progressPercent + '%', background: progressColor }"
      ></div>
    </div>
    <span class="btn-content">
      <template v-if="status === 'downloading'">
        <svg class="loading" viewBox="0 0 24 24"><circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" fill="none" stroke-dasharray="60" stroke-dashoffset="20"><animateTransform attributeName="transform" type="rotate" from="0 12 12" to="360 12 12" dur="1s" repeatCount="indefinite"/></circle></svg>
        下载中 {{ Math.round(progress * 100) }}%
      </template>
      <template v-else-if="status === 'completed'">
        已完成
      </template>
      <template v-else-if="status === 'failed'">
        重试
      </template>
      <template v-else-if="status === 'paused'">
        已暂停
      </template>
      <template v-else-if="status === 'pending'">
        等待中
      </template>
      <template v-else>
        {{ buttonText || '下载' }}
      </template>
    </span>
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
const progressColor = computed(() => {
  switch (props.status) {
    case 'downloading': return '#3498db'
    case 'completed': return '#2ecc71'
    case 'failed': return '#e74c3c'
    case 'paused': return '#f1c40f'
    default: return '#3498db'
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
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 90px;
  height: 38px;
  border: none;
  border-radius: 8px;
  font-size: 1rem;
  font-weight: 500;
  cursor: pointer;
  background: #f0f0f0;
  color: #2c3e50;
  overflow: hidden;
  transition: background 0.2s, color 0.2s;
}
.download-btn .progress-bar-bg {
  position: absolute;
  left: 0; top: 0; bottom: 0; right: 0;
  background: #e0e0e0;
  z-index: 0;
}
.download-btn .progress-bar-fill {
  position: absolute;
  left: 0; top: 0; bottom: 0;
  z-index: 1;
  transition: width 0.3s cubic-bezier(.4,0,.2,1);
  border-radius: 8px 0 0 8px;
}
.download-btn .btn-content {
  position: relative;
  z-index: 2;
  display: flex;
  align-items: center;
  gap: 0.5em;
}
.download-btn.downloading {
  background: #e3f2fd;
  color: #1976d2;
}
.download-btn.completed {
  background: #e8f5e9;
  color: #388e3c;
}
.download-btn.failed {
  background: #ffebee;
  color: #e74c3c;
}
.download-btn.paused {
  background: #fffde7;
  color: #f1c40f;
}
.download-btn.pending {
  background: #f0f0f0;
  color: #888;
}
.download-btn.disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
.loading {
  width: 1.1em;
  height: 1.1em;
  margin-right: 0.3em;
  vertical-align: middle;
}
</style> 