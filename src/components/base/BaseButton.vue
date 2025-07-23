<template>
  <button 
    :class="buttonClasses"
    :disabled="disabled || loading"
    :type="type"
    @click="handleClick"
  >
    <!-- 图标插槽 -->
    <slot name="icon" />
    
    <!-- 文字内容 -->
    <span v-if="$slots.default" class="button-text">
      <slot />
    </span>
    
    <!-- 加载状态指示器 -->
    <div v-if="loading" class="loading-spinner" />
  </button>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface Props {
  /** 按钮变体 */
  variant?: 'primary' | 'secondary' | 'danger' | 'ghost' | 'success' | 'warning'
  /** 按钮尺寸 */
  size?: 'small' | 'medium' | 'large'
  /** 是否禁用 */
  disabled?: boolean
  /** 是否加载中 */
  loading?: boolean
  /** 按钮类型 */
  type?: 'button' | 'submit' | 'reset'
  /** 是否为图标按钮 */
  iconOnly?: boolean
  /** 自定义类名 */
  class?: string
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'primary',
  size: 'medium',
  disabled: false,
  loading: false,
  type: 'button',
  iconOnly: false
})

const emit = defineEmits<{
  click: [event: MouseEvent]
}>()

const buttonClasses = computed(() => [
  'base-btn',
  `variant-${props.variant}`,
  `size-${props.size}`,
  {
    'loading': props.loading,
    'icon-only': props.iconOnly,
    'disabled': props.disabled
  },
  props.class
])

const handleClick = (event: MouseEvent) => {
  if (!props.disabled && !props.loading) {
    emit('click', event)
  }
}
</script>

<style scoped>
/* 基础按钮样式已在 desktop-design-system.css 中定义 */
/* 这里只添加组件特有的补充样式 */

.base-btn.icon-only {
  padding: var(--spacing-md);
  aspect-ratio: 1;
  min-width: auto;
}

.base-btn.size-small.icon-only {
  padding: var(--spacing-sm);
}

.base-btn.size-large.icon-only {
  padding: var(--spacing-lg);
}

.button-text {
  transition: opacity var(--transition-fast);
}

.base-btn.loading .button-text {
  opacity: 0;
}

/* 变体特殊样式 */
.base-btn.variant-success {
  background: var(--color-status-success);
  color: var(--color-text-inverse);
}

.base-btn.variant-success:hover:not(:disabled) {
  background: #059669;
  transform: translateY(-1px);
  box-shadow: var(--shadow-md);
}

.base-btn.variant-warning {
  background: var(--color-status-warning);
  color: var(--color-text-inverse);
}

.base-btn.variant-warning:hover:not(:disabled) {
  background: #d97706;
  transform: translateY(-1px);
  box-shadow: var(--shadow-md);
}

/* 焦点状态优化 */
.base-btn:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
}

.base-btn.variant-danger:focus-visible {
  outline-color: var(--color-status-error);
}

.base-btn.variant-success:focus-visible {
  outline-color: var(--color-status-success);
}

.base-btn.variant-warning:focus-visible {
  outline-color: var(--color-status-warning);
}

/* 按压效果 */
.base-btn:active:not(:disabled) {
  transform: translateY(0) scale(0.98);
}

.base-btn.variant-primary:active:not(:disabled),
.base-btn.variant-danger:active:not(:disabled),
.base-btn.variant-success:active:not(:disabled),
.base-btn.variant-warning:active:not(:disabled) {
  transform: translateY(1px) scale(0.98);
}

/* 响应式适配 */
@media (max-width: 768px) {
  .base-btn {
    padding: var(--spacing-lg) var(--spacing-xl);
    font-size: 16px;
  }
  
  .base-btn.size-small {
    padding: var(--spacing-md) var(--spacing-lg);
    font-size: 14px;
  }
  
  .base-btn.size-large {
    padding: var(--spacing-xl) var(--spacing-2xl);
    font-size: 18px;
  }
}
</style> 