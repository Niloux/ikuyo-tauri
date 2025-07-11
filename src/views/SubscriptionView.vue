<template>
  <div class="subscription-view">
    <!-- subscription-section 始终渲染，内部包含工具栏和内容区 -->
    <div class="subscription-section">
      <!-- 工具栏始终渲染 -->
      <div class="toolbar toolbar-unified">
        <div class="search-box">
          <input
            v-model="searchQuery"
            type="text"
            placeholder="搜索订阅番剧"
            class="search-input unified-input"
          />
          <span v-if="loading && searchQuery" class="search-loading-spinner" style="margin-left:8px;"></span>
        </div>
        <div class="sort-controls">
          <select
            v-model="sortOption"
            @change="handleSortOptionChange"
            class="sort-select unified-input"
          >
            <option value="subscribed_at-desc">订阅时间（降序）</option>
            <option value="subscribed_at-asc">订阅时间（升序）</option>
            <option value="rating-desc">评分（降序）</option>
            <option value="rating-asc">评分（升序）</option>
            <option value="air_date-desc">首播日期（降序）</option>
            <option value="air_date-asc">首播日期（升序）</option>
            <option value="name-asc">名称（A→Z）</option>
            <option value="name-desc">名称（Z→A）</option>
          </select>
        </div>
      </div>

      <!-- 搜索无结果提示，仅提示，内容保持不变 -->
      <div v-if="showSearchNoResult" class="search-tip" style="color: var(--color-danger, #e74c3c); margin-bottom: 1rem; text-align: left;">
        未搜索到相关订阅
      </div>

      <!-- 骨架屏加载状态 -->
      <div v-if="shouldShowSkeleton" class="anime-grid">
        <Skeleton v-for="n in 12" :key="`skeleton-${n}`" type="card" />
      </div>

      <!-- 动画卡片网格，仅在有订阅数据或搜索无结果时渲染 -->
      <div v-else-if="!loading && (subscriptions.length > 0 || showSearchNoResult)" class="anime-grid">
        <AnimeCard
          v-for="subscription in showSearchNoResult ? lastNonEmptySubscriptions : subscriptions"
          :key="subscription.bangumi_id"
          :anime="subscription.anime"
          :show-subscription-button="true"
          @click="goToDetail(subscription.anime)"
        />
      </div>

      <!-- 分页控件，仅在有数据时显示 -->
      <div v-if="!loading && (subscriptions.length > 0 || showSearchNoResult) && (showSearchNoResult ? lastNonEmptyPagination.pages : pagination.pages) > 1" class="pagination">
        <button
          @click="goToPage((showSearchNoResult ? lastNonEmptyPagination.page : pagination.page) - 1)"
          :disabled="(showSearchNoResult ? lastNonEmptyPagination.page : pagination.page) <= 1"
          class="page-btn"
        >
          上一页
        </button>

        <span class="page-info">
          {{ showSearchNoResult ? lastNonEmptyPagination.page : pagination.page }} / {{ showSearchNoResult ? lastNonEmptyPagination.pages : pagination.pages }}
        </span>

        <button
          @click="goToPage((showSearchNoResult ? lastNonEmptyPagination.page : pagination.page) + 1)"
          :disabled="(showSearchNoResult ? lastNonEmptyPagination.page : pagination.page) >= (showSearchNoResult ? lastNonEmptyPagination.pages : pagination.pages)"
          class="page-btn"
        >
          下一页
        </button>
      </div>

      <!-- 中央提示区块：无订阅且无搜索条件时显示 -->
      <div v-if="!loading && subscriptions.length === 0 && !searchQuery" class="empty-center-block">
        <h3>暂无订阅</h3>
        <p>去<router-link to="/">首页</router-link>发现你喜欢的番剧吧！</p>
      </div>

      <!-- 传统加载状态兜底（极少出现） -->
      <div v-else-if="loading" class="loading-state">
        <div class="loading-spinner"></div>
        <p>加载中...</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed, onActivated, watch } from 'vue'
import { useSubscriptionStore } from '../stores/subscriptionStore'
import AnimeCard from '../components/AnimeCard.vue'
import Skeleton from '../components/common/Skeleton.vue'
import type { BangumiCalendarItem } from '../services/bangumi/bangumiTypes'
import { useRouter } from 'vue-router'
import { ensureScrollToTop } from '../utils/scrollUtils'
import { debounce } from '../utils/debounce'

const router = useRouter()
const subscriptionStore = useSubscriptionStore()

// 响应式引用
const searchQuery = ref('')
const sortOption = ref('subscribed_at-desc')

// 新增：缓存上一次有内容的订阅和分页
const lastNonEmptySubscriptions = ref<any[]>([])
const lastNonEmptyPagination = ref<any>({ page: 1, pages: 1 })

// 新增：数据初始化标志
const isInitialized = ref(false)

// 防抖搜索
const handleSearch = debounce((val: string) => {
  subscriptionStore.searchSubscriptions(val)
}, 300)

watch(searchQuery, (val) => {
  handleSearch(val)
})

// 监听订阅数据，缓存上一次有内容的订阅和分页
watch(
  () => subscriptionStore.subscriptions,
  (subs) => {
    if (subs.length > 0) {
      lastNonEmptySubscriptions.value = subs.slice()
      lastNonEmptyPagination.value = { ...subscriptionStore.pagination }
    }
  },
  { immediate: true }
)

const handleSortOptionChange = () => {
  const [sort, order] = sortOption.value.split('-') as [
    'subscribed_at' | 'rating' | 'air_date' | 'name',
    'asc' | 'desc'
  ]
  subscriptionStore.sortSubscriptions(sort, order)
}

// 计算属性
const subscriptions = computed(() => subscriptionStore.subscriptions)
const loading = computed(() => subscriptionStore.loading)
const pagination = computed(() => subscriptionStore.pagination)

// 优化：计算属性，只有在初次加载且无订阅数据时才显示骨架屏
const shouldShowSkeleton = computed(() => {
  return loading.value && subscriptions.value.length === 0 && !searchQuery.value
})

// 新增：判断是否处于“搜索无结果但有缓存”状态
const showSearchNoResult = computed(() => {
  return searchQuery.value && subscriptions.value.length === 0 && lastNonEmptySubscriptions.value.length > 0
})

// 翻页
const goToPage = (page: number) => {
  subscriptionStore.goToPage(page)
}

const goToDetail = (anime: BangumiCalendarItem) => {
  if (anime && anime.id) {
    router.push({ name: 'anime-detail', params: { id: String(anime.id) } })
  }
}

// 数据获取
const loadSubscriptions = async () => {
  await subscriptionStore.fetchSubscriptions()
  isInitialized.value = true
}

// 页面初始化
onMounted(() => {
  ensureScrollToTop()
  loadSubscriptions()
})

onActivated(() => {
  ensureScrollToTop()
})
</script>

<style scoped>
.subscription-view {
  padding: 10px;
  max-width: 1200px;
  margin: 0 auto;
}

.page-header {
  margin-bottom: 30px;
}

.page-header h1 {
  font-size: 2.5rem;
  color: var(--color-text);
  margin: 0 0 10px 0;
}

.page-header p {
  color: var(--color-text-muted);
  margin: 0;
}

.subscription-section {
  background: var(--color-bg-white);
  border-radius: var(--radius-md, 16px);
  box-shadow: 0 2px 8px rgba(0,0,0,0.08);
  padding: 2rem;
  margin-bottom: 30px;
  transition: box-shadow 0.3s, transform 0.3s;
}
.subscription-section:hover {
  box-shadow: 0 4px 16px rgba(0,0,0,0.12);
  transform: translateY(-2px);
}

.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 30px;
  gap: 20px;
}

.toolbar-unified {
  /* background: var(--color-background-mute); */
  /* border-radius: var(--radius-md, 16px); */
  /* padding: 1rem; */
  /* box-shadow: 0 2px 8px rgba(0,0,0,0.08); */
}

.search-box {
  display: flex;
  flex: 1;
  max-width: 400px;
}

.search-input {
  flex: 1;
  padding: 10px 15px;
  border: 1px solid var(--color-border);
  border-radius: 8px 0 0 8px;
  font-size: 14px;
  outline: none;
}

.unified-input {
  border-radius: 8px;
  border: 1px solid var(--color-border);
  font-size: 14px;
  padding: 10px 15px;
  height: 40px;
}

.search-input:focus {
  border-color: var(--color-primary);
}

/* 删除无用按钮样式 */
/* .sort-order-btn { ... } 及其相关hover/active样式 */
/* .search-btn { ... } 及其相关hover样式 */

.sort-controls {
  display: flex;
  align-items: center;
  gap: 10px;
}

.sort-select {
  padding: 8px 12px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  font-size: 14px;
  outline: none;
}

.sort-select:focus {
  border-color: var(--color-primary);
}

.anime-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
  gap: 1rem;
  margin-bottom: 30px;
}

.pagination {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 20px;
  margin-top: 30px;
}

.page-btn {
  padding: 10px 20px;
  background: var(--color-background-mute);
  border: 1px solid var(--color-border);
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.3s;
  color: var(--color-text);
}

.page-btn:hover:not(:disabled) {
  background: var(--color-primary);
  color: white;
  border-color: var(--color-primary);
}

.page-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.page-info {
  color: var(--color-text-muted);
  font-size: 14px;
}

.loading-state {
  text-align: center;
  padding: 80px 20px;
}

.loading-spinner {
  width: 40px;
  height: 40px;
  border: 3px solid var(--color-background-mute);
  border-top: 3px solid var(--color-primary);
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin: 0 auto 20px;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

/* 优化：骨架屏样式 */
.skeleton-line {
  background: #e0e0e0;
  border-radius: 4px;
  animation: skeleton-loading 1.2s infinite linear;
  background: linear-gradient(90deg, #e0e0e0 25%, #f0f0f0 50%, #e0e0e0 75%);
  background-size: 200% 100%;
}

.search-skeleton {
  height: 40px;
  width: 100%;
  border-radius: 8px;
}

.sort-skeleton {
  height: 36px;
  width: 150px;
  border-radius: 6px;
}

@keyframes skeleton-loading {
  0% { background-position: 200% 0; }
  100% { background-position: -200% 0; }
}

/* 响应式设计 */
@media (max-width: 768px) {
  .anime-grid {
    grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
    gap: 0.5rem;
    margin-bottom: 1.2rem;
  }
  .subscription-section {
    padding: 0.7rem 0.2rem;
    border-radius: 8px;
    margin-bottom: 1rem;
  }
  .toolbar {
    flex-direction: column;
    gap: 10px;
    margin-bottom: 1rem;
  }
  .toolbar-unified {
    /* flex-direction: column; */
    /* gap: 10px; */
    /* padding: 0.7rem 0.2rem; */
    /* border-radius: 8px; */
  }
  .search-input {
    font-size: 1rem;
    padding: 0.7rem 0.5rem;
    border-radius: 8px 0 0 8px;
  }
  .unified-input {
    font-size: 1rem;
    padding: 0.7rem 0.5rem;
    border-radius: 8px 0 0 8px;
  }
  .search-btn {
    font-size: 1rem;
    padding: 0.7rem 0.7rem;
    border-radius: 0 8px 8px 0;
  }
  .sort-controls {
    gap: 6px;
  }
  .sort-select, .sort-order-btn {
    font-size: 0.98rem;
    padding: 0.5rem 0.7rem;
    border-radius: 6px;
  }
  .unified-input {
    font-size: 0.98rem;
    padding: 0.5rem 0.7rem;
    border-radius: 6px;
  }
  .pagination {
    gap: 8px;
    margin-top: 1rem;
  }
  .page-btn {
    font-size: 0.98rem;
    padding: 0.5rem 0.8rem;
    border-radius: 6px;
  }
  .page-info {
    font-size: 0.95rem;
  }
  .toolbar-unified {
    /* flex-direction: column; */
    /* gap: 10px; */
    /* padding: 0.7rem 0.2rem; */
    /* border-radius: 8px; */
  }
  .sort-controls {
    width: 100%;
    justify-content: flex-start;
  }
  .unified-input {
    width: 100%;
    box-sizing: border-box;
  }
}
.sort-select.unified-input {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
  font-weight: 400;
  color: #333;
  appearance: none;
  -webkit-appearance: none;
  -moz-appearance: none;
  background: #fff;
  border: 1px solid #d1d5db;
  border-radius: 12px;
  font-size: 16px;
  height: 44px;
  padding: 0 40px 0 16px;
  box-shadow: 0 1px 3px rgba(0,0,0,0.06);
  transition: border-color 0.2s, box-shadow 0.2s, background 0.2s;
  background-image: url('data:image/svg+xml;utf8,<svg fill="%23666" height="20" viewBox="0 0 20 20" width="20" xmlns="http://www.w3.org/2000/svg"><path d="M5.8 8.3a1 1 0 0 1 1.4 0L10 11.09l2.8-2.8a1 1 0 1 1 1.4 1.42l-3.5 3.5a1 1 0 0 1-1.4 0l-3.5-3.5a1 1 0 0 1 0-1.42z"/></svg>');
  background-repeat: no-repeat;
  background-position: right 14px center;
  background-size: 20px 20px;
}
.sort-select.unified-input:hover {
  background: #f5f5f7;
}
.sort-select.unified-input:focus {
  border-color: #007AFF;
  box-shadow: 0 0 0 2px rgba(0,122,255,0.12);
  outline: none;
}
.sort-select.unified-input option {
  background: #fff;
  color: #222;
}
.sort-select.unified-input option:checked {
  background: #eaf4ff;
}
@media (max-width: 768px) {
  .sort-select.unified-input {
    width: 100%;
    min-width: 0;
  }
}
/* 搜索loading动画 */
.search-loading-spinner {
  display: inline-block;
  width: 22px;
  height: 22px;
  border: 3px solid #e0e0e0;
  border-top: 3px solid var(--color-primary, #007AFF);
  border-radius: 50%;
  animation: spin 1s linear infinite;
  vertical-align: middle;
}
.empty-center-block {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 320px;
  color: var(--color-text-muted);
  text-align: center;
  padding: 60px 20px 40px 20px;
}
.empty-center-block h3 {
  font-size: 1.5rem;
  margin-bottom: 10px;
  color: var(--color-text);
}
.empty-center-block p {
  margin-bottom: 5px;
}
.empty-center-block a {
  color: var(--color-primary);
  text-decoration: none;
}
.empty-center-block a:hover {
  text-decoration: underline;
}
</style>
