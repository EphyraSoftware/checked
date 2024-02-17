<template>
  <div>
    <div v-if="loading">
      <p class="text-lg p-12">Connecting to Holochain</p>
      <span class="loading loading-infinity loading-lg"></span>
    </div>
    <div v-else>
      <div id="content">
        <DistributeGpgKey @gpg-key-dist-created="() => {}"></DistributeGpgKey>

        <MyKeys></MyKeys>

        <SearchKeys></SearchKeys>

        <Notify></Notify>
      </div>
    </div>
  </div>
</template>
<script lang="ts">
import { defineComponent, computed } from 'vue';
import { AppAgentClient, AppAgentWebsocket } from '@holochain/client';
import '@material/mwc-circular-progress';
import '@material/mwc-button';
import DistributeGpgKey from './trusted/trusted/DistributeGpgKey.vue';
import MyKeys from './trusted/trusted/MyKeys.vue';
import SearchKeys from './trusted/trusted/SearchKeys.vue';
import Notify from './component/Notify.vue';

export default defineComponent({
  components: {
    DistributeGpgKey,
    MyKeys,
    SearchKeys,
    Notify,
},
  data(): {
    client: AppAgentClient | undefined;
    loading: boolean;
  } {
    return {
      client: undefined,
      loading: true,
    };
  },
  async mounted() {
    // We pass an unused string as the url because it will dynamically be replaced in launcher environments
    this.client = await AppAgentWebsocket.connect(new URL('https://UNUSED'), 'hWOT');
    this.loading = false;
  },
  provide() {
    return {
      client: computed(() => this.client),
    };
  },
});
</script>
