<template>
  <div class="home">
    <!-- 直接渲染内容区，无需v-else -->
    <div>
      <!-- 星期导航栏 -->
      <WeekNavigation :calendar="calendar" />
      <!-- 置顶按钮 -->
      <ScrollToTopButton />
      <!-- 每日放送内容 -->
      <div class="space-y-8">
        <!-- 错误状态 -->
        <Alert v-if="error && !loading" variant="destructive" class="mb-6">
          <AlertDescription class="flex items-center justify-between">
            <span>{{ error }}</span>
            <Button
              @click="loadCalendar"
              variant="outline"
              size="sm"
              class="ml-4"
            >
              重试
            </Button>
          </AlertDescription>
        </Alert>

        <!-- 优化：骨架屏加载状态 -->
        <template v-if="shouldShowSkeleton">
          <div v-for="n in 3" :key="`skeleton-${n}`" class="mb-8">
            <Card class="animate-pulse">
              <CardHeader>
                <div class="flex items-center gap-3">
                  <UiSkeleton class="h-8 w-24" />
                  <UiSkeleton class="h-5 w-12 rounded-full" />
                </div>
              </CardHeader>
              <CardContent>
                <div
                  class="grid grid-cols-[repeat(auto-fill,minmax(200px,1fr))] gap-4"
                >
                  <div
                    v-for="m in 6"
                    :key="`skeleton-card-${m}`"
                    class="space-y-3"
                  >
                    <UiSkeleton class="aspect-[3/4] w-full rounded-lg" />
                    <div class="space-y-2">
                      <UiSkeleton class="h-4 w-full" />
                      <UiSkeleton class="h-3 w-3/4" />
                      <UiSkeleton class="h-3 w-1/2" />
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>
          </div>
        </template>

        <!-- 数据内容 -->
        <template v-else-if="!error">
          <div
            v-for="(day, dayIndex) in calendar"
            :key="day.weekday.id"
            :id="`day-${day.weekday.id}`"
            class="mb-8"
          >
            <Card
              class="transition-all duration-300 hover:-translate-y-1 hover:shadow-lg"
            >
              <CardHeader>
                <div class="flex items-center gap-3">
                  <CardTitle class="text-2xl">
                    {{ day.weekday.cn }}
                  </CardTitle>
                  <Badge
                    v-if="isToday(day.weekday.id)"
                    variant="destructive"
                    class="text-xs"
                  >
                    今天
                  </Badge>
                </div>
              </CardHeader>
              <CardContent>
                <div
                  class="grid grid-cols-[repeat(auto-fill,minmax(200px,1fr))] gap-4"
                >
                  <AnimeCard
                    v-for="(anime, animeIndex) in day.items"
                    :key="anime.id"
                    :anime="anime"
                    @click="goToDetail(anime.id)"
                    @image-load="() => {}"
                  />
                </div>
              </CardContent>
            </Card>
          </div>
        </template>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
export default {
  name: "HomeView",
};
</script>

<script setup lang="ts">
import { onMounted, onActivated, computed } from "vue";
import { useRouter } from "vue-router";
import { storeToRefs } from "pinia";
import { useHomeStore } from "../stores/homeStore";
import { useSubscriptionStore } from "../stores/subscriptionStore";
import AnimeCard from "../components/AnimeCard.vue";
import WeekNavigation from "../components/WeekNavigation.vue";
import ScrollToTopButton from "../components/ScrollToTopButton.vue";
import BangumiApiService from "../services/bangumi/bangumiApiService";
import { ensureScrollToTop } from "../utils/scrollUtils";
import { onBeforeRouteLeave } from "vue-router";

// 导入shadcn-vue组件
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton as UiSkeleton } from "@/components/ui/skeleton";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";

const router = useRouter();
const homeStore = useHomeStore();
const subscriptionStore = useSubscriptionStore();

// 从store获取响应式状态
const { loading, error, cachedCalendar, hasCalendarData } =
  storeToRefs(homeStore);

// 优化：计算属性，只有在真正需要加载且无数据时才显示骨架屏
const shouldShowSkeleton = computed(() => {
  return loading.value && !hasCalendarData.value;
});

// 缓存今天的weekdayId，避免重复计算
const todayWeekdayId = computed(() => {
  const today = new Date().getDay();
  return today === 0 ? 7 : today;
});

// 优化：判断是否是今天
const isToday = (weekdayId: number): boolean => {
  const adjustedWeekdayId = weekdayId === 0 ? 7 : weekdayId;
  return adjustedWeekdayId === todayWeekdayId.value;
};

// 优化：获取距离今天的天数差
const getDaysFromToday = (weekdayId: number): number => {
  const adjustedWeekdayId = weekdayId === 0 ? 7 : weekdayId;
  let diff = adjustedWeekdayId - todayWeekdayId.value;
  if (diff < 0) {
    diff += 7;
  }
  return diff;
};

// 优化：使用computed自动排序日历数据，避免重复计算
const calendar = computed(() => {
  if (!cachedCalendar.value.length) return [];

  return [...cachedCalendar.value].sort((a, b) => {
    if (isToday(a.weekday.id)) return -1;
    if (isToday(b.weekday.id)) return 1;
    return getDaysFromToday(a.weekday.id) - getDaysFromToday(b.weekday.id);
  });
});

// 优化：简化加载逻辑，避免状态冲突
const loadCalendar = async () => {
  try {
    homeStore.loading = true;
    homeStore.error = null;

    const data = await BangumiApiService.getCalendar();
    // 直接存储原始数据，排序由computed处理
    homeStore.setCalendarData(data);
  } catch (err) {
    console.error("加载每日放送失败:", err);
    homeStore.error = "加载失败，请检查网络连接或API服务状态";
  } finally {
    homeStore.loading = false;
  }
};

// 跳转到番剧详情页
const goToDetail = (bangumiId: number) => {
  router.push(`/anime/${bangumiId}`);
};

// 路由守卫：离开页面时保存滚动位置
onBeforeRouteLeave((to, from) => {
  const currentScrollPosition =
    window.pageYOffset || document.documentElement.scrollTop;
  homeStore.saveScrollPosition(currentScrollPosition);

  // 如果是去往详情页，设置sessionStorage标记
  if (to.name === "anime-detail" || to.name === "library-detail") {
    sessionStorage.setItem("fromDetail", "true");
  } else {
    // 去往其他页面，清除标记
    sessionStorage.removeItem("fromDetail");
  }
});

// keep-alive组件恢复时的处理
onActivated(() => {
  const fromDetail = sessionStorage.getItem("fromDetail");

  if (fromDetail === "true") {
    // 从详情页返回，立即恢复滚动位置
    sessionStorage.removeItem("fromDetail");
    const savedPosition = homeStore.getScrollPosition();
    window.scrollTo({ top: savedPosition, behavior: "instant" });
  } else {
    // 从其他页面返回，滚动到顶部
    ensureScrollToTop();
  }
});

// 组件挂载时加载数据
onMounted(() => {
  // 首次挂载时加载数据，滚动位置管理由keep-alive + onActivated处理
  if (!hasCalendarData.value) {
    loadCalendar();
  }
  // 拉取全部订阅ID，确保首页订阅状态准确
  subscriptionStore.fetchAllSubscriptionIds();
});
</script>
