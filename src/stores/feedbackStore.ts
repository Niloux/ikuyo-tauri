// 全局交互反馈Store：统一管理全局loading、error状态
// 用于全局Loading遮罩、全局错误弹窗（Error）
// 通过Pinia store实现，所有页面和组件均可直接调用
import { defineStore } from "pinia";

export const useFeedbackStore = defineStore("feedback", {
  state: () => ({
    loading: false as boolean, // 全局Loading遮罩状态
    error: "" as string | null, // 全局Error弹窗内容
    // 延迟loading定时器
    _loadingTimer: null as ReturnType<typeof setTimeout> | null,
    // 确认对话框状态
    confirmDialog: {
      show: false,
      message: "",
      resolve: null as ((value: boolean) => void) | null,
    },
  }),
  actions: {
    // 显示全局Loading遮罩（延迟150ms）
    showLoading() {
      if (this._loadingTimer) return;
      this._loadingTimer = setTimeout(() => {
        this.loading = true;
        this._loadingTimer = null;
      }, 150);
    },
    // 隐藏全局Loading遮罩
    hideLoading() {
      if (this._loadingTimer) {
        clearTimeout(this._loadingTimer);
        this._loadingTimer = null;
      }
      this.loading = false;
    },
    // 显示确认对话框
    showConfirm(message: string): Promise<boolean> {
      this.confirmDialog.message = message;
      this.confirmDialog.show = true;
      return new Promise((resolve) => {
        this.confirmDialog.resolve = resolve;
      });
    },
    // 解决确认对话框
    resolveConfirm(result: boolean) {
      if (this.confirmDialog.resolve) {
        this.confirmDialog.resolve(result);
      }
      this.confirmDialog.show = false;
      this.confirmDialog.message = "";
      this.confirmDialog.resolve = null;
    },
  },
});
