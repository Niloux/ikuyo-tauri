<template>
  <svg
    :width="size"
    :height="size"
    :class="['icon', `icon-${name}`, { 'icon-hover': hover }]"
    :style="{ color }"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    :stroke-width="strokeWidth"
    stroke-linecap="round"
    stroke-linejoin="round"
  >
    <!-- folder-open 打开目录 -->
    <g v-if="name === 'folder-open'">
      <path d="M3 7v10a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-6l-2-2H5a2 2 0 0 0-2 2z"/>
      <path d="M3 7h18"/>
    </g>

    <!-- pause 暂停 -->
    <g v-else-if="name === 'pause'">
      <rect x="6" y="4" width="4" height="16" rx="1"/>
      <rect x="14" y="4" width="4" height="16" rx="1"/>
    </g>

    <!-- play 播放 -->
    <g v-else-if="name === 'play'">
      <polygon points="5,3 19,12 5,21"/>
    </g>

    <!-- more-horizontal 更多操作 -->
    <g v-else-if="name === 'more-horizontal'">
      <circle cx="12" cy="12" r="1"/>
      <circle cx="19" cy="12" r="1"/>
      <circle cx="5" cy="12" r="1"/>
    </g>

    <!-- search 搜索 -->
    <g v-else-if="name === 'search'">
      <circle cx="11" cy="11" r="8"/>
      <path d="m21 21-4.35-4.35"/>
    </g>

    <!-- check-circle 已完成 -->
    <g v-else-if="name === 'check-circle'">
      <circle cx="12" cy="12" r="10"/>
      <path d="m9 12 2 2 4-4"/>
    </g>

    <!-- x-circle 失败 -->
    <g v-else-if="name === 'x-circle'">
      <circle cx="12" cy="12" r="10"/>
      <path d="m15 9-6 6"/>
      <path d="m9 9 6 6"/>
    </g>

    <!-- clock 等待 -->
    <g v-else-if="name === 'clock'">
      <circle cx="12" cy="12" r="10"/>
      <polyline points="12,6 12,12 16,14"/>
    </g>

    <!-- folder 文件夹 -->
    <g v-else-if="name === 'folder'">
      <path d="M4 20h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.93a2 2 0 0 1-1.66-.9l-.82-1.2A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13c0 1.1.9 2 2 2Z"/>
    </g>

    <!-- trash 删除 -->
    <g v-else-if="name === 'trash'">
      <path d="M3 6h18"/>
      <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/>
      <path d="M8 6V4c0-1 1-2 2-2h4c0-1 1-2 2-2v2"/>
      <line x1="10" y1="11" x2="10" y2="17"/>
      <line x1="14" y1="11" x2="14" y2="17"/>
    </g>

    <!-- film 空状态 -->
    <g v-else-if="name === 'film'">
      <rect x="2" y="3" width="20" height="18" rx="2"/>
      <path d="M7 3v18"/>
      <path d="M17 3v18"/>
      <path d="M2 9h20"/>
      <path d="M2 15h20"/>
      <circle cx="12" cy="12" r="2"/>
    </g>

    <!-- download 下载 -->
    <g v-else-if="name === 'download'">
      <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
      <polyline points="7,10 12,15 17,10"/>
      <line x1="12" y1="15" x2="12" y2="3"/>
    </g>

    <!-- refresh 刷新 -->
    <g v-else-if="name === 'refresh'">
      <path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8"/>
      <path d="M21 3v5h-5"/>
      <path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16"/>
      <path d="M3 21v-5h5"/>
    </g>

    <!-- retry 重试 -->
    <g v-else-if="name === 'retry'">
      <path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8"/>
      <path d="M21 3v5h-5"/>
    </g>
  </svg>
</template>

<script setup lang="ts">
interface Props {
  name: string
  size?: number | string
  color?: string
  strokeWidth?: number
  hover?: boolean
}

withDefaults(defineProps<Props>(), {
  size: 20,
  color: 'currentColor',
  strokeWidth: 2,
  hover: false
})
</script>

<style scoped>
.icon {
  display: inline-block;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  vertical-align: middle;
}

.icon-hover:hover {
  transform: scale(1.1);
}

/* 特定图标的悬停效果 */
.icon-trash.icon-hover:hover {
  color: #ef4444;
  transform: scale(1.1);
}

.icon-play.icon-hover:hover,
.icon-pause.icon-hover:hover {
  color: #3b82f6;
  transform: scale(1.1);
}

.icon-check-circle.icon-hover:hover {
  color: #10b981;
}

.icon-x-circle.icon-hover:hover {
  color: #ef4444;
}

.icon-clock.icon-hover:hover {
  color: #f59e0b;
}

.icon-folder.icon-hover:hover,
.icon-folder-open.icon-hover:hover {
  color: #8b5cf6;
}

.icon-search.icon-hover:hover {
  color: #06b6d4;
}

.icon-more-horizontal.icon-hover:hover {
  color: #6b7280;
}

.icon-film.icon-hover:hover {
  color: #8b5cf6;
}

.icon-download.icon-hover:hover {
  color: #10b981;
}

.icon-refresh.icon-hover:hover,
.icon-retry.icon-hover:hover {
  color: #f59e0b;
  animation: spin 0.8s ease-in-out;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}
</style> 