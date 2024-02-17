<script setup lang="ts">
import { ref, watch } from "vue";
import { useThemeStore, Theme } from "../store/theme-store";

const themeStore = useThemeStore();

const themeToggle = ref(!themeStore.isDefault);

watch(themeToggle, (newVal) => {
  console.log("set theme", newVal);
  if (newVal) {
    console.log("call set theme");
    themeStore.setTheme(Theme.Luxury);
  } else {
    themeStore.setTheme(Theme.Bumblebee);
  }
});
</script>

<template>
  <div class="form-control">
    <label class="label cursor-pointer">
      <span class="label-text">Dark mode</span>
      <input type="checkbox" class="toggle" v-model="themeToggle" />
    </label>
  </div>
</template>
