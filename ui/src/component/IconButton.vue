<script setup lang="ts">
defineProps<{
  title: string;
  icon: string; // Passed straight to font-awesome-icon
}>();

defineEmits<{
  (e: "click"): void;
}>();
</script>

<template>
  <button
    class="btn btn-circle btn-sm btn-primary"
    :title="title"
    @click="$emit('click')"
  >
    <font-awesome-icon :icon="icon" />
  </button>
</template>
