import { defineStore } from "pinia";
import { computed, ref } from "vue";

interface Notification {
  id?: string;
  message: string;
  type: "info" | "warning" | "error";
  timeout?: number;
}

export const useNotificationsStore = defineStore("notifications", () => {
  const id = ref(0);
  const notifications = ref<{ [x: string]: Notification }>({});

  const pushNotification = (notification: Notification) => {
    const use_id = (id.value++).toString();

    if (id.value > 1000) {
      // Reset the id to avoid eventual overflow
      id.value = 0;
    }

    notification.id = use_id;
    notifications.value[use_id] = notification;
    setTimeout(() => {
      delete notifications.value[use_id];
    }, notification.timeout ?? 3000);
  };

  const orderedNotifications = computed(() => {
    return Object.keys(notifications.value)
      .sort()
      .map((key) => {
        return notifications.value[key];
      });
  });

  return {
    notifications,
    orderedNotifications,
    pushNotification,
  };
});
