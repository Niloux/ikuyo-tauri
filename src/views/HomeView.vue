<template>
  <div class="home">
    <!-- 星期导航栏 -->
    <WeekNavigation :calendar="calendar" />
    
    <!-- 置顶按钮 -->
    <ScrollToTopButton />
    
    <!-- 每日放送内容 -->
    <div class="calendar-container">
      <!-- 错误状态 -->
      <BaseCard v-if="error && !loading" class="error-state">
        <div class="error-content">
          <div class="error-icon">
            <Icon name="x-circle" :size="48" color="var(--color-status-error)" />
          </div>
          <h3>加载失败</h3>
          <p>{{ error }}</p>
          <BaseButton 
            variant="primary" 
            @click="loadCalendar"
            class="retry-button"
          >
            <template #icon>
              <Icon name="refresh" :size="16" />
            </template>
            重试
          </BaseButton>
        </div>
      </BaseCard>

      <!-- 骨架屏加载状态 -->
      <template v-else-if="shouldShowSkeleton">
        <BaseCard
          v-for="n in 7" 
          :key="`skeleton-${n}`" 
          class="day-section"
          hover
        >
          <template #header>
            <div class="skeleton-day-title">
              <div class="skeleton-line" style="width: 60px; height: 24px;"></div>
            </div>
          </template>
          <div class="anime-grid">
            <Skeleton v-for="m in 6" :key="`skeleton-card-${m}`" type="card" />
          </div>
        </BaseCard>
      </template>

      <!-- 数据内容 -->
      <template v-else>
        <BaseCard
          v-for="(day, dayIndex) in calendar"
          :key="day.weekday.id"
          :id="`day-${day.weekday.id}`"
          class="day-section"
          hover
        >
          <template #header>
            <h2 class="day-title" :class="{ 'today': isToday(day.weekday.id) }">
              {{ day.weekday.cn }}
              <span v-if="isToday(day.weekday.id)" class="today-badge">今天</span>
            </h2>
          </template>
          
          <div class="anime-grid">
            <AnimeCard
              v-for="(anime, animeIndex) in day.items"
              :key="anime.id"
              :anime="anime"
              @click="goToDetail(anime.id)"
              @image-load="() => {}"
            />
          </div>
        </BaseCard>
      </template>
    </div>
  </div>
</template>

<script lang="ts">
export default {
  name: 'HomeView'
}
</script>

<script setup lang="ts">
import { ref, onMounted, onActivated, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { storeToRefs } from 'pinia'
import { useHomeStore } from '../stores/homeStore'
import { useSubscriptionStore } from '../stores/subscriptionStore'
import AnimeCard from '../components/AnimeCard.vue'
import WeekNavigation from '../components/WeekNavigation.vue'
import ScrollToTopButton from '../components/ScrollToTopButton.vue'
import Skeleton from '../components/common/Skeleton.vue'
import { BaseCard, BaseButton } from '../components/base'
import Icon from '../components/common/Icon.vue'
import BangumiApiService from '../services/bangumi/bangumiApiService'
import type { BangumiWeekday } from '../services/bangumi/bangumiTypes'
import { ensureScrollToTop } from '../utils/scrollUtils'
import { onBeforeRouteLeave } from 'vue-router'

const route = useRoute()
const router = useRouter()
const homeStore = useHomeStore()
const subscriptionStore = useSubscriptionStore()

// 从store获取响应式状态
const { loading, error, cachedCalendar, hasCalendarData } = storeToRefs(homeStore)

// 优化：计算属性，只有在真正需要加载且无数据时才显示骨架屏
const shouldShowSkeleton = computed(() => {
  return loading.value && !hasCalendarData.value
})

// 缓存今天的weekdayId，避免重复计算
const todayWeekdayId = computed(() => {
  const today = new Date().getDay()
  return today === 0 ? 7 : today
})

// 优化：判断是否是今天
const isToday = (weekdayId: number): boolean => {
  const adjustedWeekdayId = weekdayId === 0 ? 7 : weekdayId
  return adjustedWeekdayId === todayWeekdayId.value
}

// 优化：获取距离今天的天数差
const getDaysFromToday = (weekdayId: number): number => {
  const adjustedWeekdayId = weekdayId === 0 ? 7 : weekdayId
  let diff = adjustedWeekdayId - todayWeekdayId.value
  if (diff < 0) {
    diff += 7
  }
  return diff
}

// 优化：使用computed自动排序日历数据，避免重复计算
const calendar = computed(() => {
  if (!cachedCalendar.value.length) return []

  return [...cachedCalendar.value].sort((a, b) => {
    if (isToday(a.weekday.id)) return -1
    if (isToday(b.weekday.id)) return 1
    return getDaysFromToday(a.weekday.id) - getDaysFromToday(b.weekday.id)
  })
})

// 优化：简化加载逻辑，避免状态冲突
const loadCalendar = async () => {
  try {
    homeStore.loading = true
    homeStore.error = null

    const data = await BangumiApiService.getCalendar()
    // 直接存储原始数据，排序由computed处理
    homeStore.setCalendarData(data)
  } catch (err) {
    console.error('加载每日放送失败:', err)
    homeStore.error = '加载失败，请检查网络连接或API服务状态'
  } finally {
    homeStore.loading = false
  }
}

// 跳转到番剧详情页
const goToDetail = (bangumiId: number) => {
  router.push(`/anime/${bangumiId}`)
}

// 路由守卫：离开页面时保存滚动位置
onBeforeRouteLeave((to, from) => {
  const currentScrollPosition = window.pageYOffset || document.documentElement.scrollTop
  homeStore.saveScrollPosition(currentScrollPosition)

  // 如果是去往详情页，设置sessionStorage标记
  if (to.name === 'anime-detail' || to.name === 'library-detail') {
    sessionStorage.setItem('fromDetail', 'true')
  } else {
    // 去往其他页面，清除标记
    sessionStorage.removeItem('fromDetail')
  }
})

// keep-alive组件恢复时的处理
onActivated(() => {
  const fromDetail = sessionStorage.getItem('fromDetail')

  if (fromDetail === 'true') {
    // 从详情页返回，立即恢复滚动位置
    sessionStorage.removeItem('fromDetail')
    const savedPosition = homeStore.getScrollPosition()
    window.scrollTo({ top: savedPosition, behavior: 'instant' })
  } else {
    // 从其他页面返回，滚动到顶部
    ensureScrollToTop()
  }
})

// 组件挂载时加载数据
onMounted(() => {
  // 首次挂载时加载数据，滚动位置管理由keep-alive + onActivated处理
  if (!hasCalendarData.value) {
    loadCalendar()
  }
  // 拉取全部订阅ID，确保首页订阅状态准确
  subscriptionStore.fetchAllSubscriptionIds()
})
</script>

<style scoped>
/* 使用新的桌面设计系统 */
.home {
  padding: 0;
  max-width: 1200px;
  margin: 0 auto;
}

/* 错误状态样式 */
.error-state {
  margin: var(--spacing-2xl) 0;
}

.error-content {
  text-align: center;
  padding: var(--spacing-2xl);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--spacing-lg);
}

.error-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0.8;
}

.error-content h3 {
  color: var(--color-text-primary);
  font-size: 1.5rem;
  font-weight: 600;
  margin: 0;
}

.error-content p {
  color: var(--color-text-secondary);
  margin: 0;
  max-width: 400px;
  line-height: 1.6;
}

/* 日历容器 */
.calendar-container {
  display: flex;
  flex-direction: column;
  gap: var(--section-gap);
}

/* 骨架屏样式 */
.skeleton-day-title {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.skeleton-line {
  background: var(--color-bg-tertiary);
  border-radius: var(--radius-sm);
  animation: skeleton-loading 1.2s infinite linear;
  background: linear-gradient(90deg, 
    var(--color-bg-tertiary) 25%, 
    var(--color-bg-hover) 50%, 
    var(--color-bg-tertiary) 75%);
  background-size: 200% 100%;
}

@keyframes skeleton-loading {
  0% { background-position: 200% 0; }
  100% { background-position: -200% 0; }
}

/* 日标题样式 */
.day-title {
  font-size: 1.5rem;
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0;
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
}

.day-title.today {
  color: var(--color-status-error);
}

.today-badge {
  background: var(--color-status-error);
  color: var(--color-text-inverse);
  padding: var(--spacing-xs) var(--spacing-sm);
  border-radius: var(--radius-xl);
  font-size: 12px;
  font-weight: 600;
  box-shadow: var(--shadow-sm);
}

/* 动画网格 */
.anime-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
  gap: var(--component-gap);
}

/* 响应式设计 */
@media (max-width: 768px) {
  .home {
    padding: var(--spacing-sm);
  }
  
  .calendar-container {
    gap: var(--spacing-xl);
  }
  
  .anime-grid {
    grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
    gap: var(--spacing-md);
  }
  
  .day-title {
    font-size: 1.25rem;
    gap: var(--spacing-sm);
  }
  
  .today-badge {
    font-size: 11px;
    padding: 2px var(--spacing-sm);
  }
}
</style>
