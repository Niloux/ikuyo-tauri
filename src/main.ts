import "./styles/globals.css";

import { createApp } from "vue";
import { createPinia } from "pinia";
import { toast } from "vue-sonner";

import App from "./App.vue";
import router from "./router";

const app = createApp(App);

app.use(createPinia());
app.use(router);

app.config.errorHandler = (err, vm, info) => {
  // 处理错误，例如：报告给错误监控服务
  console.error("Vue 应用错误:", err, vm, info);
  toast.error("应用发生错误，请刷新页面或联系管理员。");
};

if ("requestIdleCallback" in window) {
  requestIdleCallback(() => {
    // 预取次要页面和组件chunk
    import("./views/ResourceLibraryView.vue");
    import("./components/AnimeCard.vue");
    import("./components/TaskModal.vue");
  });
} else {
  setTimeout(() => {
    import("./views/ResourceLibraryView.vue");
    import("./components/AnimeCard.vue");
    import("./components/TaskModal.vue");
  }, 2000);
}

app.mount("#app");
