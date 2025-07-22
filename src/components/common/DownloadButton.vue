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
      <span class="button-text" :class="{ 'text-animate': animateText }" :style="{ color: textColor }">{{ buttonText }}</span>
    </button>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useDownloadStore } from '@/stores/downloadStore'

const props = defineProps<{
  resourceId: number
  onAction?: (action: 'download' | 'pause' | 'resume' | 'delete' | 'retry' | 'play') => void
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

// 文字颜色根据状态平滑过渡
const textColor = computed(() => {
  switch (uiState.value.status) {
    case 'downloading': return '#1976d2'
    case 'completed': return '#fff'
    case 'failed': return '#fff'
    case 'paused': return '#b26a00'
    case 'pending': return '#888'
    default: return '#333'
  }
})

const animateText = ref(false)
let animateTimeout: number | null = null

// watch状态变化，触发文字缩放动画
watch(() => uiState.value.status, () => {
  animateText.value = false
  if (animateTimeout) window.clearTimeout(animateTimeout)
  // 触发重绘
  void containerRef.value?.offsetWidth
  animateText.value = true
  animateTimeout = window.setTimeout(() => {
    animateText.value = false
  }, 500) // 动画时长与CSS一致
})

const handleClick = (e: Event) => {
  if (disabled.value) return
  if (uiState.value.status === 'downloading') {
    props.onAction && props.onAction('pause')
    return
  }
  let action: 'download' | 'pause' | 'resume' | 'delete' | 'retry' | 'play'
  switch (uiState.value.status) {
    case 'completed': action = 'play'; break
    case 'failed': action = 'retry'; break
    case 'paused': action = 'resume'; break
    default: action = 'download'; break
  }
  props.onAction && props.onAction(action)
}
</script>

<style scoped>
/* DownloadButton 动画优化，macOS风格流畅过渡（无渐变，纯色背景） */
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
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 0;
  /* 精细化transition，提升状态切换流畅度 */
  transition: background-color 0.35s cubic-bezier(.4,0,.2,1),
              color 0.35s cubic-bezier(.4,0,.2,1),
              box-shadow 0.35s cubic-bezier(.4,0,.2,1),
              opacity 0.35s cubic-bezier(.4,0,.2,1),
              transform 0.18s cubic-bezier(.4,0,.2,1);
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
  /* 进度条动画优化 */
  transition: width 0.35s cubic-bezier(.4,0,.2,1),
              background 0.35s cubic-bezier(.4,0,.2,1),
              border-radius 0.35s cubic-bezier(.4,0,.2,1),
              opacity 0.25s cubic-bezier(.4,0,.2,1);
  opacity: 1;
}

.download-btn > * {
  position: relative;
  z-index: 2;
}

/* downloading - 主色蓝 纯色背景 */
.download-btn.downloading {
  background-color: #e3f2fd;
  color: #1976d2;
  box-shadow: 0 2px 8px rgba(25,118,210,0.08);
}
.download-btn.downloading::before {
  background: #1976d2;
  opacity: 0.15;
}

/* completed - 绿色 纯色背景 */
.download-btn.completed {
  background-color: #43a047;
  color: #fff;
  box-shadow: 0 2px 8px rgba(67,160,71,0.10);
}
.download-btn.completed::before {
  background: #fff;
  opacity: 0.18;
}

/* failed - 红色 纯色背景 */
.download-btn.failed {
  background-color: #e74c3c;
  color: #fff;
  box-shadow: 0 2px 8px rgba(231,76,60,0.10);
}
.download-btn.failed::before {
  background: #fff;
  opacity: 0.18;
}

/* paused - 橙色 纯色背景 */
.download-btn.paused {
  background-color: #fff3cd;
  color: #b26a00;
  box-shadow: 0 2px 8px rgba(178,106,0,0.08);
}
.download-btn.paused::before {
  background: #b26a00;
  opacity: 0.15;
}

/* pending - 灰色 纯色背景 */
.download-btn.pending {
  background-color: #f0f0f0;
  color: #888;
  box-shadow: 0 2px 8px rgba(189,189,189,0.08);
}
.download-btn.pending::before {
  background: #bdbdbd;
  opacity: 0.15;
}

/* disabled - 透明度降低，进度条隐藏 */
.download-btn.disabled {
  opacity: 0.6;
  cursor: not-allowed;
  /* 禁用时色彩、背景、阴影平滑淡出 */
}
.download-btn.disabled::before {
  opacity: 0;
}

/* 其他状态进度条不显示 */
.download-btn:not(.downloading):not(.completed):not(.failed):not(.paused):not(.pending)::before {
  opacity: 0;
}

/* 悬浮与点击微交互动画 */
.download-btn:not(.disabled):hover {
  transform: scale(1.03);
  box-shadow: 0 4px 16px rgba(25, 118, 210, 0.12);
}
.download-btn:not(.disabled):active {
  transform: scale(0.97);
  box-shadow: 0 1px 2px rgba(25, 118, 210, 0.10);
}

/* failed状态点击时轻微shake动画 */
@keyframes shake {
  0% { transform: translateX(0); }
  20% { transform: translateX(-2px); }
  40% { transform: translateX(2px); }
  60% { transform: translateX(-2px); }
  80% { transform: translateX(2px); }
  100% { transform: translateX(0); }
}
.download-btn.failed:active {
  animation: shake 0.3s cubic-bezier(.4,0,.2,1);
}

/* prefers-reduced-motion: 动效降级 */
@media (prefers-reduced-motion: reduce) {
  .download-btn,
  .download-btn::before {
    transition-duration: 0.1s !important;
    animation-duration: 0.1s !important;
  }
}
.button-text {
  display: inline-block;
  transition: color 0.5s cubic-bezier(.4,0,.2,1), transform 0.5s cubic-bezier(.4,0,.2,1);
}
.button-text.text-animate {
  animation: text-bounce 0.5s cubic-bezier(.4,0,.2,1);
}
@keyframes text-bounce {
  0% { transform: scale(1); }
  20% { transform: scale(1.08); }
  60% { transform: scale(0.96); }
  80% { transform: scale(1.04); }
  100% { transform: scale(1); }
}
@media (prefers-reduced-motion: reduce) {
  .button-text {
    transition-duration: 0.1s !important;
  }
  .button-text.text-animate {
    animation-duration: 0.1s !important;
  }
}
</style> 