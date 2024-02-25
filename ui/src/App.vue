<script setup lang="ts">
import { ref, provide, onMounted } from "vue";
import { AppAgentClient, AppAgentWebsocket } from "@holochain/client";
import DistributeGpgKey from "./trusted/trusted/DistributeGpgKey.vue";
import MyKeys from "./trusted/trusted/MyKeys.vue";
import SearchKeys from "./trusted/trusted/SearchKeys.vue";
import NotifyContainer from "./component/NotifyContainer.vue";
import { useThemeStore } from "./store/theme-store";
import SettingsEditor from "./component/SettingsEditor.vue";
import KeyCollections from "./trusted/trusted/KeyCollections.vue";
import { useRouter } from "vue-router";

const themeStore = useThemeStore();
const router = useRouter();

const client = ref<AppAgentClient | null>(null);
provide("client", client);
const loading = ref(true);
const showScreen = ref<"home" | "search" | "settings" | "about">("home");

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
  client.value = await AppAgentWebsocket.connect(
    new URL("https://UNUSED"),
    "hWOT",
  );
  loading.value = false;
});

// TODO Would be nice if this worked, but really Holochain needs to detect dropped connections and stop trying to broadcast on them
// window.onbeforeunload = () => {
//   if (client.value) {
//     (client.value as AppAgentWebsocket).appWebsocket.client.close();
//   }
// };
</script>

<template>
  <div>
    <div v-if="loading">
      <p class="text-lg p-12">Connecting to Holochain</p>
      <span class="loading loading-infinity loading-lg"></span>
    </div>
    <div v-else>
      <div class="navbar bg-base-100">
        <div class="navbar-start">
          <div class="dropdown">
            <div tabindex="0" role="button" class="btn btn-ghost btn-circle">
              <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24"
                stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h7" />
              </svg>
            </div>
            <ul tabindex="0" class="menu menu-sm dropdown-content mt-3 z-[1] p-2 shadow bg-base-100 rounded-box w-52">
              <li><router-link to="/">Home</router-link></li>
              <li>
                <router-link to="/settings">Settings</router-link>
              </li>
              <li><router-link to="/about">About</router-link></li>
            </ul>
          </div>
        </div>
        <div class="navbar-center">
          <button class="btn btn-ghost text-xl" @click="router.push('/')">
            Web of Trust
          </button>
        </div>
        <div class="navbar-end">
          <router-link to="/search" class="px-2">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
          </router-link>
        </div>
      </div>

      <router-view></router-view>

      <NotifyContainer></NotifyContainer>
    </div>
  </div>
</template>
