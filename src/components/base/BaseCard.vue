<template>
  <div :class="cardClasses">
    <!-- 卡片头部 -->
    <div v-if="$slots.header" class="card-header">
      <slot name="header" />
    </div>
    
    <!-- 卡片主体内容 -->
    <div class="card-content">
      <slot />
    </div>
    
    <!-- 卡片底部 -->
    <div v-if="$slots.footer" class="card-footer">
      <slot name="footer" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface Props {
  /** 卡片变体 */
  variant?: 'default' | 'elevated' | 'flat'
  /** 是否启用悬停效果 */
  hover?: boolean
  /** 是否为紧凑模式 */
  compact?: boolean
  /** 自定义类名 */
  class?: string
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'default',
  hover: true,
  compact: false
})

const cardClasses = computed(() => [
  'base-card',
  `variant-${props.variant}`,
  {
    'hover': props.hover,
    'compact': props.compact
  },
  props.class
])
</script>

<style scoped>
/* 基础卡片样式已在 desktop-design-system.css 中定义 */
/* 这里只添加组件特有的补充样式 */

.base-card.compact .card-content {
  padding: var(--spacing-lg);
}

.base-card.compact .card-header,
.base-card.compact .card-footer {
  padding: var(--spacing-lg);
}

/* 响应式适配 */
@media (max-width: 768px) {
  .card-content {
    padding: var(--spacing-lg);
  }
  
  .card-header,
  .card-footer {
    padding: var(--spacing-lg);
  }
}
</style> 