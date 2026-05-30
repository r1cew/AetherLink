<template>
  <TransitionGroup
    name="notification"
    tag="div"
    class="notifications-container"
  >
    <div
      v-for="notification in notifications"
      :key="notification.id"
      :class="['notification', `notification-${notification.type}`]"
      @click="removeNotification(notification.id)"
    >
      <span class="notification-icon">
        {{ getIcon(notification.type) }}
      </span>
      <span class="notification-message">{{ notification.message }}</span>
      <button
        class="notification-close"
        @click.stop="removeNotification(notification.id)"
      >
        ✕
      </button>
    </div>
  </TransitionGroup>
</template>

<script setup lang="ts">
import { useNotification } from "../../composables/useNotification";

const { notifications, removeNotification } = useNotification();

const getIcon = (type: string) => {
  const icons = {
    success: "✓",
    error: "✗",
    warning: "⚠",
    info: "☞",
  };
  return icons[type as keyof typeof icons] || "";
};
</script>

<style scoped>
.notifications-container {
  position: fixed;
  top: 20px;
  right: 20px;
  z-index: 9999;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.notification {
  min-width: 280px;
  padding: 12px 16px;
  border-radius: var(--radius);
  background: var(--bg2);
  border: 1px solid var(--border);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  display: flex;
  align-items: center;
  gap: 12px;
  cursor: pointer;
  transition: all 0.3s ease;
  backdrop-filter: blur(10px);
}

.notification-success {
  border-left: 4px solid var(--status3);
  color: #a4e0a4;
}

.notification-error {
  border-left: 4px solid var(--status2);
  color: #ff9e9e;
}

.notification-warning {
  border-left: 4px solid var(--accent2);
  color: #ffdd9e;
}

.notification-info {
  border-left: 4px solid var(--text2);
  color: var(--text);
}

.notification-icon {
  font-size: 18px;
  font-weight: 600;
}

.notification-message {
  font-size: 14px;
  flex: 1;
}

.notification:hover {
  transform: translateX(-4px);
  background: var(--bg3);
}

.notification-close {
  margin-left: auto;
  background: none;
  border: none;
  cursor: pointer;
  opacity: 0.5;
  font-size: 16px;
  color: var(--text);
  padding: 4px;
  border-radius: 4px;
  transition: all 0.2s ease;
}

.notification-close:hover {
  opacity: 1;
  background: rgba(255, 255, 255, 0.1);
}

/* Анимации */
.notification-enter-active,
.notification-leave-active {
  transition: all 0.3s ease;
}

.notification-enter-from {
  opacity: 0;
  transform: translateX(30px);
}

.notification-leave-to {
  opacity: 0;
  transform: translateX(30px);
}
</style>
