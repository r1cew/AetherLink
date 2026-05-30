import { ref, readonly } from "vue";

type NotificationType = "success" | "error" | "info" | "warning";

interface Notification {
  id: number;
  message: string;
  type: NotificationType;
  duration?: number;
}

const notifications = ref<Notification[]>([]);
let nextId = 0;

export const useNotification = () => {
  const showNotification = (
    message: string,
    type: NotificationType = "info",
    duration = 3000,
  ) => {
    const id = nextId++;
    notifications.value.push({ id, message, type, duration });

    setTimeout(() => {
      notifications.value = notifications.value.filter((n) => n.id !== id);
    }, duration);
  };

  const removeNotification = (id: number) => {
    notifications.value = notifications.value.filter((n) => n.id !== id);
  };

  return {
    notifications: readonly(notifications),
    showNotification,
    removeNotification,
    success: (msg: string, duration?: number) =>
      showNotification(msg, "success", duration),
    error: (msg: string, duration?: number) =>
      showNotification(msg, "error", duration),
    info: (msg: string, duration?: number) =>
      showNotification(msg, "info", duration),
    warning: (msg: string, duration?: number) =>
      showNotification(msg, "warning", duration),
  };
};
