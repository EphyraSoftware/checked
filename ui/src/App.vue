<script setup lang="ts">
import { ref, provide, onMounted } from "vue";
import { AppAgentClient, AppAgentWebsocket } from "@holochain/client";
import DistributeGpgKey from "./trusted/trusted/DistributeGpgKey.vue";
import MyKeys from "./trusted/trusted/MyKeys.vue";
import SearchKeys from "./trusted/trusted/SearchKeys.vue";
import NotifyContainer from "./component/NotifyContainer.vue";
import { useThemeStore } from "./store/theme-store";
import SettingsEditor from "./component/SettingsEditor.vue";

const themeStore = useThemeStore();

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
              <svg
                xmlns="http://www.w3.org/2000/svg"
                class="h-5 w-5"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M4 6h16M4 12h16M4 18h7"
                />
              </svg>
            </div>
            <ul
              tabindex="0"
              class="menu menu-sm dropdown-content mt-3 z-[1] p-2 shadow bg-base-100 rounded-box w-52"
            >
              <li><a @click="showScreen = 'home'">Home</a></li>
              <li><a @click="showScreen = 'settings'">Settings</a></li>
              <li><a @click="showScreen = 'about'">About</a></li>
            </ul>
          </div>
        </div>
        <div class="navbar-center">
          <button class="btn btn-ghost text-xl" @click="showScreen = 'home'">
            Web of Trust
          </button>
        </div>
        <div class="navbar-end">
          <button
            class="btn btn-ghost btn-circle"
            @click="showScreen = 'search'"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              class="h-5 w-5"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
              />
            </svg>
          </button>
        </div>
      </div>

      <div v-if="showScreen === 'home'">
        <div class="container mx-auto mt-5">
          <MyKeys></MyKeys>
          <div class="mt-5">
            <DistributeGpgKey
              @gpg-key-dist-created="() => {}"
            ></DistributeGpgKey>
          </div>
        </div>
      </div>
      <div v-else-if="showScreen === 'settings'">
        <div class="container mx-auto w-1/2 mt-5">
          <SettingsEditor></SettingsEditor>
        </div>
      </div>
      <div v-else-if="showScreen === 'search'">
        <div class="container mx-auto mt-5">
          <SearchKeys></SearchKeys>
        </div>
      </div>
      <div v-else-if="showScreen === 'about'">
        <div class="container mx-auto mt-5">
          <p>
            You can learn more about protecting your E-mails with GPG from the
            <a
              href="https://emailselfdefense.fsf.org/en/"
              target="_blank"
              class="link"
              >Free Software Foundation</a
            >.
          </p>
          <p>
            This hApp is a distributed solution for section #5. Rather than
            needing to send your public keys to a centralized server, you can
            distribute them using Holochain.
          </p>

          <br />
          <p>
            You can get the
            <a
              href="https://github.com/ThetaSinner/hWOT"
              target="_blank"
              class="link"
              >source code</a
            >
            for this app to verify its behavior.
          </p>
        </div>
      </div>

      <NotifyContainer></NotifyContainer>
    </div>
  </div>
</template>
