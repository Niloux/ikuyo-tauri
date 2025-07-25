<template>
  <button
    class="subscription-btn"
    :class="[
      sizeClass,
      { subscribed: isSubscribed, loading: subscriptionLoading },
    ]"
    :disabled="subscriptionLoading"
    :title="isSubscribed ? '取消订阅' : '订阅'"
    @click.stop="handleSubscriptionToggle"
  >
    <span v-if="subscriptionLoading">
      <Icon name="clock" :size="20" color="var(--color-status-warning)" />
    </span>
    <span v-else>
      <svg
        v-if="!isSubscribed"
        width="22"
        height="22"
        viewBox="0 0 24 24"
        fill="none"
        stroke="#e50914"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <path
          d="M16.5 3c-1.74 0-3.41 1.01-4.5 2.09C10.91 4.01 9.24 3 7.5 3 4.42 3 2 5.42 2 8.5c0 3.78 3.4 6.86 8.55 11.54a2 2 0 0 0 2.9 0C18.6 15.36 22 12.28 22 8.5 22 5.42 19.58 3 16.5 3z"
        />
      </svg>
      <svg
        v-else
        width="22"
        height="22"
        viewBox="0 0 24 24"
        fill="#e50914"
        stroke="#e50914"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <path
          d="M16.5 3c-1.74 0-3.41 1.01-4.5 2.09C10.91 4.01 9.24 3 7.5 3 4.42 3 2 5.42 2 8.5c0 3.78 3.4 6.86 8.55 11.54a2 2 0 0 0 2.9 0C18.6 15.36 22 12.28 22 8.5 22 5.42 19.58 3 16.5 3z"
        />
      </svg>
      <span v-if="showText" class="btn-text">{{
        isSubscribed ? "已订阅" : "订阅"
      }}</span>
    </span>
  </button>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import { useSubscriptionStore } from "../../stores/subscriptionStore";
import type { BangumiCalendarItem } from "../../services/bangumi/bangumiTypes";
import Icon from "./Icon.vue";
import { toast } from "vue-sonner";

const props = withDefaults(
  defineProps<{
    anime: BangumiCalendarItem;
    size?: "small" | "medium" | "large";
    showText?: boolean;
  }>(),
  {
    size: "medium",
    showText: false,
  },
);

const subscriptionStore = useSubscriptionStore();
const subscriptionLoading = ref(false);

const isSubscribed = computed(() =>
  subscriptionStore.isSubscribed(props.anime.id),
);

const sizeClass = computed(() => {
  switch (props.size) {
    case "small":
      return "subscription-btn--small";
    case "large":
      return "subscription-btn--large";
    default:
      return "subscription-btn--medium";
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
.subscription-btn {
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: none;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.92);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  cursor: pointer;
  transition:
    transform 0.18s cubic-bezier(0.4, 1.3, 0.6, 1),
    box-shadow 0.18s;
  padding: 0;
  outline: none;
  user-select: none;
}
.subscription-btn--small {
  width: 28px;
  height: 28px;
}
.subscription-btn--medium {
  width: 36px;
  height: 36px;
}
.subscription-btn--large {
  width: 48px;
  height: 48px;
}
.subscription-btn:hover:not(:disabled) {
  transform: scale(1.12);
  box-shadow: 0 4px 16px rgba(229, 9, 20, 0.18);
}
.subscription-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
.subscription-btn svg {
  display: block;
}
.subscription-btn.subscribed {
  background: rgba(229, 9, 20, 0.1);
}
.btn-text {
  margin-left: 0.5em;
  font-size: 1rem;
  color: #e50914;
  font-weight: 600;
}
</style>
