<script setup lang="ts">
import { RouterView, useRouter } from 'vue-router'
import AppLayout from './components/AppLayout.vue'
import { onMounted } from 'vue'
import { useDownloadStore } from '@/stores/downloadStore'
import { Toaster } from '@/components/ui/sonner'
import 'vue-sonner/style.css'

const router = useRouter()
const downloadStore = useDownloadStore()

onMounted(() => {
  downloadStore.init()
})
</script>

<template>
  <AppLayout>
    <Toaster class="pointer-events-auto" />
    <RouterView v-slot="{ Component, route }">
      <transition name="fade-page" mode="out-in">
        <keep-alive v-if="route.meta.keepAlive">
          <component :is="Component" :key="route.fullPath" />
        </keep-alive>
        <component v-else :is="Component" :key="route.fullPath" />
      </transition>
    </RouterView>
  </AppLayout>
</template>

<style scoped>
</style>
