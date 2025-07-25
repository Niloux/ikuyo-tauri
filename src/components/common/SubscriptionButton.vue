<template>
  <Button
    :size="buttonSize"
    :disabled="subscriptionLoading"
    :title="isSubscribed ? '取消订阅' : '订阅'"
    @click.stop="handleSubscriptionToggle"
    :variant="isSubscribed ? 'secondary' : 'default'"
  >
    <template v-if="subscriptionLoading">
      <Loader :size="20" class="animate-spin" />
    </template>
    <template v-else>
      <Heart
        :size="20"
        color="#e50914"
        :fill="isSubscribed ? '#e50914' : 'none'"
        :stroke="'#e50914'"
      />
    </template>
  </Button>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import { useSubscriptionStore } from "../../stores/subscriptionStore";
import type { BangumiCalendarItem } from "../../services/bangumi/bangumiTypes";
import { toast } from "vue-sonner";
import Button from "../ui/button/Button.vue";
import { Heart, Loader } from "lucide-vue-next";

const props = withDefaults(
  defineProps<{
    anime: BangumiCalendarItem;
    size?: "small" | "medium" | "large";
  }>(),
  {
    size: "medium",
  },
);

const subscriptionStore = useSubscriptionStore();
const subscriptionLoading = ref(false);

const isSubscribed = computed(() =>
  subscriptionStore.isSubscribed(props.anime.id),
);

const buttonSize = computed(() => {
  switch (props.size) {
    case "small":
      return "sm";
    case "large":
      return "lg";
    default:
      return "default";
  }
});

const handleSubscriptionToggle = async () => {
  try {
    subscriptionLoading.value = true;
    await subscriptionStore.toggleSubscription(props.anime);
  } catch (error) {
    toast.error("订阅操作失败，请重试");
    console.error("订阅操作失败:", error);
  } finally {
    subscriptionLoading.value = false;
  }
};
</script>

<style scoped>
</style>
