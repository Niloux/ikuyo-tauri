// IKuYo 桌面软件基础组件库
// 基于下载管理页面的优秀设计，统一的桌面软件风格组件

export { default as BaseCard } from './BaseCard.vue'
export { default as BaseButton } from './BaseButton.vue'
export { default as BaseTabs } from './BaseTabs.vue'

// 组件类型导出
export type { default as BaseCardProps } from './BaseCard.vue'
export type { default as BaseButtonProps } from './BaseButton.vue'
export type { default as BaseTabsProps } from './BaseTabs.vue'

// 常用类型定义
export interface Tab {
    key: string
    label: string
    count?: number
    icon?: string
    disabled?: boolean
    closable?: boolean
} 