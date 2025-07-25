<template>
  <Card 
    class="group overflow-hidden aspect-[1/1.618] transition-all duration-300 hover:shadow-lg hover:-translate-y-1 active:scale-[0.98] cursor-pointer flex flex-col p-0"
    @click="handleCardClick" 
    ref="cardRef"
  >
    <!-- 番剧封面 -->
    <div class="relative w-full h-full overflow-hidden bg-muted">
      <!-- 订阅按钮 -->
      <Button
        v-if="props.showSubscriptionButton"
        variant="secondary"
        size="icon"
        class="absolute top-2 left-2 z-20 w-9 h-9 rounded-full bg-background/90 backdrop-blur-sm hover:bg-background shadow-sm transition-all duration-300 hover:scale-110 active:scale-95"
        @click.stop="handleSubscriptionClick"
      >
        <Heart 
          :class="[
            'w-4 h-4 transition-colors',
            isSubscribed ? 'fill-red-500 text-red-500' : 'text-muted-foreground'
          ]"
        />
      </Button>

      <!-- 评分徽章 -->
      <Badge 
        v-if="props.anime.rating && props.anime.rating.score > 0"
        class="absolute top-2 right-2 z-20 bg-orange-500 text-white font-semibold px-2 py-1 flex items-center gap-1"
      >
        <Star class="w-3 h-3 fill-current" />
        {{ props.anime.rating.score.toFixed(1) }}
      </Badge>

      <!-- 图片 -->
      <img
        v-if="shouldLoadImage"
        :src="imageUrl"
        :alt="props.anime.name_cn || props.anime.name"
        @error="onImageError"
        @load="$emit('imageLoad')"
        class="w-full h-full object-cover object-top transition-transform duration-300 group-hover:scale-105"
      />
      <Skeleton v-else class="w-full h-full" />
    </div>

    <!-- 番剧信息 -->
    <CardContent class="flex-1 p-4 flex flex-col min-h-28">
      <CardTitle class="line-clamp-1 h-6 text-base font-semibold mb-1 leading-tight">
        {{ props.anime.name_cn || props.anime.name }}
      </CardTitle>
      
      <CardDescription 
        class="line-clamp-1 h-5 text-sm text-muted-foreground mb-3"
      >
        {{ (props.anime.name_cn && props.anime.name !== props.anime.name_cn) ? props.anime.name : '' }}
      </CardDescription>

      <div class="mt-auto flex items-center justify-between text-sm text-muted-foreground">
        <div class="flex items-center gap-1">
          <Calendar class="w-4 h-4" />
          <span>{{ formattedAirDate }}</span>
        </div>
        
        <div 
          v-if="props.anime.rating && props.anime.rating.total > 0"
          class="flex items-center gap-1"
        >
          <Users class="w-4 h-4" />
          <span>{{ formatRatingCount(props.anime.rating.total) }}</span>
        </div>
      </div>
    </CardContent>
  </Card>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { Heart, Star, Calendar, Users } from 'lucide-vue-next'
import type { BangumiCalendarItem } from '../services/bangumi/bangumiTypes'
import defaultCover from '../assets/ikuyo-avatar.png'
import { createLazyObserver } from '../utils/lazyLoad'

// shadcn-vue 组件导入
import { Card, CardContent, CardTitle, CardDescription } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Skeleton } from '@/components/ui/skeleton'

// Props定义
const props = withDefaults(defineProps<{
  anime: BangumiCalendarItem
  showSubscriptionButton?: boolean
  isSubscribed?: boolean
}>(), {
  showSubscriptionButton: true,
  isSubscribed: false
})

// Events定义
const emit = defineEmits<{
  click: []
  imageLoad: []
  subscribe: []
}>()

// 懒加载本地状态
const shouldLoadImage = ref(false)
const cardRef = ref<any>(null)
let observer: IntersectionObserver | null = null

// 格式化播出日期
const formattedAirDate = computed(() => {
  const dateStr = props.anime.air_date
  if (!dateStr) return '未知'

  try {
    const date = new Date(dateStr)
    return date.toLocaleDateString('zh-CN', {
      month: 'short',
      day: 'numeric'
    })
  } catch {
    return dateStr
  }
})

// 格式化评价人数
const formatRatingCount = (count: number): string => {
  if (count >= 10000) {
    return `${Math.floor(count / 10000)}万人`
  } else if (count >= 1000) {
    return `${Math.floor(count / 1000)}k人`
  }
  return `${count}人`
}

// 图片URL处理
const imageUrl = computed(() => {
  const imgObj = props.anime.images
  if (!imgObj?.large) return defaultCover
  return imgObj.large.replace(/^http:/, 'https:')
})

// 图片加载失败处理
const onImageError = (event: Event) => {
  const img = event.target as HTMLImageElement
  img.src = defaultCover
}

// 卡片点击处理
const handleCardClick = () => {
  emit('click')
}

// 订阅按钮点击处理
const handleSubscriptionClick = (event: Event) => {
  event.stopPropagation()
  emit('subscribe')
}

// 懒加载初始化
onMounted(() => {
  if (cardRef.value?.$el) {
    observer = createLazyObserver(cardRef.value.$el, () => {
      shouldLoadImage.value = true
    })
  }
})

onUnmounted(() => {
  if (observer) {
    observer.disconnect()
    observer = null
  }
})
</script>

<style scoped>
/* 多行文本截断工具类 */
.line-clamp-1 {
  display: -webkit-box;
  -webkit-line-clamp: 1;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.line-clamp-2 {
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

/* 响应式调整 */
@media (max-width: 768px) {
  .aspect-\[3\/4\] {
    min-height: 160px;
  }
}
</style>