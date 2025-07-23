<template>
  <div class="base-tabs">
    <!-- 标签页头部 -->
    <div class="tabs-header">
      <div class="tabs-nav">
        <div 
          v-for="tab in tabs" 
          :key="tab.key"
          :class="['tab-item', { active: activeTab === tab.key, disabled: tab.disabled }]"
          @click="!tab.disabled && setActiveTab(tab.key)"
          @keydown.enter="!tab.disabled && setActiveTab(tab.key)"
          @keydown.space.prevent="!tab.disabled && setActiveTab(tab.key)"
          tabindex="0"
          role="tab"
          :aria-selected="activeTab === tab.key"
          :aria-disabled="tab.disabled"
        >
          <!-- 图标 -->
          <slot v-if="tab.icon" :name="`tab-icon-${tab.key}`">
            <span class="tab-icon">{{ tab.icon }}</span>
          </slot>
          
          <!-- 标签文字 -->
          <span class="tab-label">{{ tab.label }}</span>
          
          <!-- 计数徽章 -->
          <span v-if="tab.count !== undefined && tab.count !== null" class="tab-count">
            {{ formatCount(tab.count) }}
          </span>
          
          <!-- 关闭按钮 -->
          <button 
            v-if="tab.closable" 
            class="tab-close"
            @click.stop="closeTab(tab.key)"
            tabindex="-1"
            aria-label="关闭标签页"
          >
            ×
          </button>
        </div>
      </div>
      
      <!-- 右侧操作区 -->
      <div v-if="$slots.actions" class="tabs-actions">
        <slot name="actions" />
      </div>
    </div>
    
    <!-- 标签页内容 -->
    <div class="tabs-content">
      <slot :activeTab="activeTab" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, watch } from 'vue'

interface Tab {
  key: string
  label: string
  count?: number
  icon?: string
  disabled?: boolean
  closable?: boolean
}

interface Props {
  /** 标签页列表 */
  tabs: Tab[]
  /** 当前激活的标签页 */
  modelValue?: string
  /** 是否允许键盘导航 */
  keyboardNavigation?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  keyboardNavigation: true
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
  'tab-change': [key: string]
  'tab-close': [key: string]
}>()

const activeTab = computed({
  get: () => props.modelValue || (props.tabs.length > 0 ? props.tabs[0].key : ''),
  set: (value: string) => {
    emit('update:modelValue', value)
  }
})

const setActiveTab = (key: string) => {
  const tab = props.tabs.find(t => t.key === key)
  if (tab && !tab.disabled) {
    activeTab.value = key
    emit('tab-change', key)
  }
}

const closeTab = (key: string) => {
  emit('tab-close', key)
}

const formatCount = (count: number): string => {
  if (count > 999) {
    return '999+'
  }
  return count.toString()
}

// 监听 modelValue 变化
watch(() => props.modelValue, (newValue) => {
  if (newValue && newValue !== activeTab.value) {
    setActiveTab(newValue)
  }
}, { immediate: true })
</script>

<style scoped>
/* 基础标签页样式已在 desktop-design-system.css 中定义 */
/* 这里只添加组件特有的补充样式 */

.tabs-nav {
  display: flex;
  align-items: center;
  flex: 1;
  overflow-x: auto;
  scrollbar-width: none;
  -ms-overflow-style: none;
}

.tabs-nav::-webkit-scrollbar {
  display: none;
}

.tabs-actions {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  margin-left: var(--spacing-lg);
}

.tab-item {
  flex-shrink: 0;
  outline: none;
  user-select: none;
}

.tab-item:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: -2px;
  border-radius: var(--radius-sm);
}

.tab-item.disabled {
  opacity: 0.5;
  cursor: not-allowed;
  color: var(--color-text-disabled);
}

.tab-item.disabled:hover {
  color: var(--color-text-disabled);
}

.tab-icon {
  font-size: 16px;
  line-height: 1;
}

.tab-label {
  white-space: nowrap;
}

.tab-close {
  background: none;
  border: none;
  color: var(--color-text-tertiary);
  cursor: pointer;
  font-size: 16px;
  line-height: 1;
  padding: 0;
  margin-left: var(--spacing-xs);
  border-radius: var(--radius-sm);
  width: 16px;
  height: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all var(--transition-fast);
}

.tab-close:hover {
  background: var(--color-status-error-light);
  color: var(--color-status-error);
}

.tab-close:focus-visible {
  outline: 1px solid var(--color-status-error);
  outline-offset: 1px;
}

/* 移动端适配 */
@media (max-width: 768px) {
  .tabs-header {
    flex-direction: column;
    gap: var(--spacing-md);
  }
  
  .tabs-nav {
    order: 1;
    width: 100%;
  }
  
  .tabs-actions {
    order: 0;
    margin-left: 0;
    justify-content: center;
    width: 100%;
  }
  
  .tab-item {
    margin-right: var(--spacing-lg);
    padding: var(--spacing-md) 0;
  }
  
  .tab-item:last-child {
    margin-right: 0;
  }
}

/* 暗色主题适配 */
@media (prefers-color-scheme: dark) {
  .tab-close:hover {
    background: rgba(239, 68, 68, 0.2);
  }
}
</style> 