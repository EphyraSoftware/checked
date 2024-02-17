<script setup lang="ts">
import { useNotificationsStore } from "../store/notifications-store";

const notifications = useNotificationsStore();
</script>

<template>
  <div class="toast toast-end">
    <div v-for="n in notifications.orderedNotifications" v-bind:key="n.id">
      <div
        :class="{
          alert: true,
          'alert-info': n.type === 'info',
          'alert-warning': n.type === 'warning',
          'alert-error': n.type === 'error',
        }"
      >
        <span>{{ n.message }}</span>
      </div>
    </div>
  </div>
</template>
