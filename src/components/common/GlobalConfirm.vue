<template>
  <transition name="fade">
    <div v-if="confirmDialog.show" class="global-confirm-overlay" @click.self="cancel">
      <transition name="slide-up">
        <div v-if="confirmDialog.show" class="confirm-dialog">
          <div class="dialog-header">
            <h3 class="dialog-title">操作确认</h3>
            <button class="close-button" @click="cancel">&times;</button>
          </div>
          <div class="dialog-body">
            <p>{{ confirmDialog.message }}</p>
          </div>
          <div class="dialog-footer">
            <button class="btn btn-secondary" @click="cancel">取消</button>
            <button class="btn btn-primary" @click="confirm">确认</button>
          </div>
        </div>
      </transition>
    </div>
  </transition>
</template>

<script setup lang="ts">
import { useFeedbackStore } from '@/stores/feedbackStore';
import { storeToRefs } from 'pinia';

const feedbackStore = useFeedbackStore();
const { confirmDialog } = storeToRefs(feedbackStore);

const confirm = () => {
  feedbackStore.resolveConfirm(true);
};

const cancel = () => {
  feedbackStore.resolveConfirm(false);
};
</script>

<style scoped>
.global-confirm-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 1000;
}

.confirm-dialog {
  background: white;
  border-radius: 8px;
  box-shadow: 0 5px 15px rgba(0,0,0,0.3);
  width: 90%;
  max-width: 400px;
  overflow: hidden;
}

.dialog-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1rem 1.5rem;
  border-bottom: 1px solid #eee;
}

.dialog-title {
  margin: 0;
  font-size: 1.25rem;
  font-weight: 600;
  color: #333;
}

.close-button {
  background: none;
  border: none;
  font-size: 1.5rem;
  cursor: pointer;
  color: #aaa;
}

.dialog-body {
  padding: 1.5rem;
  font-size: 1rem;
  color: #555;
  line-height: 1.6;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  padding: 1rem 1.5rem;
  background-color: #f9f9f9;
  border-top: 1px solid #eee;
}

.btn {
  padding: 0.6rem 1.2rem;
  border: none;
  border-radius: 5px;
  font-size: 0.9rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-secondary {
  background-color: #f0f0f0;
  color: #555;
  margin-right: 0.5rem;
}

.btn-secondary:hover {
  background-color: #e0e0e0;
}

.btn-primary {
  background-color: var(--color-primary);
  color: white;
}

.btn-primary:hover {
  background-color: var(--color-primary-dark);
}

/* Transitions */
.fade-enter-active, .fade-leave-active {
  transition: opacity 0.3s ease;
}
.fade-enter-from, .fade-leave-to {
  opacity: 0;
}

.slide-up-enter-active, .slide-up-leave-active {
  transition: transform 0.3s ease, opacity 0.3s ease;
}
.slide-up-enter-from, .slide-up-leave-to {
  opacity: 0;
  transform: translateY(30px);
}
</style>
