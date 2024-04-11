<script setup lang="ts">
import { ref, provide, onMounted } from "vue";
import { AppAgentClient, AppAgentWebsocket } from "@holochain/client";
import NotifyContainer from "./component/NotifyContainer.vue";
import { useThemeStore } from "./store/theme-store";

const themeStore = useThemeStore();

const client = ref<AppAgentClient | null>(null);
provide("client", client);
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
  client.value = await AppAgentWebsocket.connect("hWOT");
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
