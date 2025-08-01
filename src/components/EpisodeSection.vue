<template>
    <div class="episode-display">
      <!-- 加载状态 -->
      <div v-if="loading" class="loading-state">
        <p>正在加载章节信息...</p>
      </div>

      <!-- 错误状态 -->
      <div v-else-if="error" class="error-state">
        <p>{{ error || '加载失败' }}</p>
      </div>

      <!-- 根据集数智能选择展示方式 -->
      <EpisodeList
        v-else-if="episodeStats"
        :display-mode="displayMode"
        :bangumi-id="props.bangumiId"
        :total-episodes="episodeStats.main_episodes"
        :bangumi-episodes="episodes"
        :preloaded-availability="availability"
        :loading="loading"
        :error="error"
        :subject="props.subject"
      />


    </div>
  </template>

  <script setup lang="ts">
  import { computed } from 'vue'
  import { storeToRefs } from 'pinia'
  import { useAnimeDetailStore } from '../stores/animeDetailStore'
  import EpisodeList from './EpisodeList.vue'

  // Props定义
  interface Props {
    bangumiId: number
    subject: any // 可细化类型
  }

  const props = defineProps<Props>()

  const animeDetailStore = useAnimeDetailStore()
  const { episodes, availability } = storeToRefs(animeDetailStore)
  const loading = computed(() => animeDetailStore.fetchAllAsync.loading)
  const error = computed(() => animeDetailStore.fetchAllAsync.error)

  // 本地统计信息接口定义
  interface EpisodeStats {
    total: number
    main_episodes: number
    special_episodes: number
    opening_episodes: number
    ending_episodes: number
    pv_episodes: number
    other_episodes: number
  }

  

  // 统计信息
  const episodeStats = computed<EpisodeStats | null>(() => {
    if (!episodes.value) return null
    return {
      total: episodes.value.length,
      main_episodes: episodes.value.filter(ep => ep.type === 0).length,
      special_episodes: episodes.value.filter(ep => ep.type === 1).length,
      opening_episodes: episodes.value.filter(ep => ep.type === 2).length,
      ending_episodes: episodes.value.filter(ep => ep.type === 3).length,
      pv_episodes: episodes.value.filter(ep => ep.type === 4).length,
      other_episodes: episodes.value.filter(ep => ep.type === 6).length
    }
  })

  // 智能显示模式判断
  const MODERN_ANIME_THRESHOLD = 26

  const displayMode = computed(() => {
    if (!episodeStats.value) return 'carousel'
    return episodeStats.value.main_episodes <= MODERN_ANIME_THRESHOLD ? 'carousel' : 'grid'
  })

  // 组件挂载时获取数据
  // 这里不需要手动获取数据，因为数据已经通过pinia存储
  </script>

  <style scoped>
  .episode-display {
    /* 容器样式，让子组件决定具体样式 */
  }

  .loading-state, .error-state {
    text-align: center;
    padding: 2rem;
    color: #6c757d;
  }

  .error-state {
    color: #dc3545;
  }

  .batch-progress {
    margin-top: 1rem;
  }

  .progress-bar {
    width: 200px;
    height: 6px;
    background-color: #e9ecef;
    border-radius: 3px;
    margin: 0.5rem auto;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background-color: #3498db;
    transition: width 0.3s ease;
  }

  </style>
