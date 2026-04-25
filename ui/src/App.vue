<script setup lang="ts">
import { ref, onMounted } from "vue";
import { AppWebsocket } from "@holochain/client";
import NotifyContainer from "./component/NotifyContainer.vue";
import { useThemeStore } from "./store/theme-store";
import { holochainClient } from "./holochain-client";

const themeStore = useThemeStore();

const loading = ref(true);

const applyTheme = (theme: string) => {
  document.documentElement.setAttribute("data-theme", theme);
};

onMounted(async () => {
  // Set the current theme on load
  applyTheme(themeStore.theme);
  // then listen for changes to the theme and apply them
  themeStore.$subscribe((_, state) => {
    applyTheme(state.theme);
  });

  // We pass an unused string as the url because it will dynamically be replaced in launcher environments
  holochainClient.value = await AppWebsocket.connect();
  loading.value = false;
});
</script>

<template>
  <div>
    <div v-if="loading">
      <p class="text-lg p-12">Connecting to Holochain</p>
      <span class="loading loading-infinity loading-lg"></span>
    </div>
    <div v-else>
      <router-view></router-view>

      <NotifyContainer></NotifyContainer>
    </div>
  </div>
</template>
