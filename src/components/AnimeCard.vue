<template>
  <Card
    class="group flex max-h-[280px] min-h-[200px] cursor-pointer flex-col overflow-hidden p-0 transition-all duration-300 hover:-translate-y-1 hover:shadow-lg active:scale-[0.98] md:max-h-[320px] md:min-h-[240px] lg:max-h-[400px] lg:min-h-[280px]"
    @click="handleCardClick"
    ref="cardRef"
  >
    <!-- 番剧封面 -->
    <div class="bg-muted relative h-[66.7%] max-h-[240px] overflow-hidden">
      <!-- 订阅按钮 -->
      <SubscriptionButton
        v-if="props.showSubscriptionButton"
        :anime="props.anime"
        :isSubscribed="props.isSubscribed"
        size="small"
        :showText="false"
        class="bg-background/90 hover:bg-background absolute top-2 left-2 z-20 h-9 w-9 rounded-full shadow-sm backdrop-blur-sm transition-all duration-300 hover:scale-110 active:scale-95"
        @click.stop="handleSubscriptionClick"
      />

      <!-- 评分徽章 -->
      <Badge
        v-if="props.anime.rating && props.anime.rating.score > 0"
        class="absolute top-2 right-2 z-20 flex items-center gap-1 bg-orange-500 px-2 py-1 font-semibold text-white"
      >
        <Star class="h-3 w-3 fill-current" />
        {{ props.anime.rating.score.toFixed(1) }}
      </Badge>

      <!-- 图片 -->
      <img
        v-if="shouldLoadImage"
        :src="imageUrl"
        :alt="props.anime.name_cn || props.anime.name"
        @error="onImageError"
        @load="$emit('imageLoad')"
        class="h-full w-full object-cover object-top transition-transform duration-300 group-hover:scale-105"
      />
      <Skeleton v-else class="h-full w-full" />
    </div>

    <!-- 番剧信息 -->
    <CardContent class="flex min-h-[100px] flex-1 flex-col px-4 pt-3 pb-4">
      <CardTitle
        class="mb-2 line-clamp-2 text-base leading-tight font-semibold"
      >
        {{ props.anime.name_cn || props.anime.name }}
      </CardTitle>

      <CardDescription class="text-muted-foreground mb-3 line-clamp-1 text-sm">
        {{ originalName }}
      </CardDescription>

      <div
        class="text-muted-foreground mt-auto flex items-center justify-between text-sm"
      >
        <div class="flex items-center gap-1">
          <Calendar class="h-4 w-4" />
          <span>{{ formattedAirDate }}</span>
        </div>

        <div
          v-if="props.anime.rating && props.anime.rating.total > 0"
          class="flex items-center gap-1"
        >
          <Users class="h-4 w-4" />
          <span>{{ formatRatingCount(props.anime.rating.total) }}</span>
        </div>
      </div>
    </CardContent>
  </Card>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { Star, Calendar, Users } from "lucide-vue-next";
import type { BangumiCalendarItem } from "../services/bangumi/bangumiTypes";
import defaultCover from "../assets/ikuyo-avatar.png";
import { createLazyObserver } from "../utils/lazyLoad";

// shadcn-vue 组件导入
import {
  Card,
  CardContent,
  CardTitle,
  CardDescription,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import SubscriptionButton from "./common/SubscriptionButton.vue";

// Props定义
const props = withDefaults(
  defineProps<{
    anime: BangumiCalendarItem;
    showSubscriptionButton?: boolean;
    isSubscribed?: boolean;
  }>(),
  {
    showSubscriptionButton: true,
    isSubscribed: false,
  },
);

// Events定义
const emit = defineEmits<{
  click: [];
  imageLoad: [];
  subscribe: [];
}>();

// 懒加载本地状态
const shouldLoadImage = ref(false);
const cardRef = ref<any>(null);
let observer: IntersectionObserver | null = null;

// 计算属性
const originalName = computed(() => {
  if (props.anime.name_cn && props.anime.name !== props.anime.name_cn) {
    return props.anime.name;
  }
  return "";
});

const formattedAirDate = computed(() => {
  const dateStr = props.anime.air_date;
  if (!dateStr) return "未知";

  try {
    const date = new Date(dateStr);
    return date.toLocaleDateString("zh-CN", {
      month: "short",
      day: "numeric",
    });
  } catch {
    return dateStr;
  }
});

const imageUrl = computed(() => {
  const imgObj = props.anime.images;
  if (!imgObj?.large) return defaultCover;
  return imgObj.large.replace(/^http:/, "https:");
});

// 方法
const formatRatingCount = (count: number): string => {
  if (count >= 10000) {
    return `${Math.floor(count / 10000)}万人`;
  } else if (count >= 1000) {
    return `${Math.floor(count / 1000)}k人`;
  }
  return `${count}人`;
};

const onImageError = (event: Event) => {
  const img = event.target as HTMLImageElement;
  img.src = defaultCover;
};

const handleCardClick = () => {
  emit("click");
};

const handleSubscriptionClick = (event: Event) => {
  event.stopPropagation();
  emit("subscribe");
};

// 生命周期钩子
onMounted(() => {
  if (cardRef.value?.$el) {
    observer = createLazyObserver(cardRef.value.$el, () => {
      shouldLoadImage.value = true;
    });
  }
});

onUnmounted(() => {
  if (observer) {
    observer.disconnect();
    observer = null;
  }
});
</script>

<style scoped>
/* 样式保持为空，因为所有样式都由Tailwind处理 */
</style>
