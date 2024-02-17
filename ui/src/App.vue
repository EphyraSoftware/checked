<template>
  <div>
    <div v-if="loading">
      <mwc-circular-progress indeterminate></mwc-circular-progress>
    </div>
    <div v-else>
      <div id="content" style="display: flex; flex-direction: column; flex: 1;">
        <CreateGpgKey @gpg-key-dist-created="() => {}"></CreateGpgKey>

        <MyKeys></MyKeys>

        <SearchKeys></SearchKeys>
      </div>
    </div>
  </div>
</template>
<script lang="ts">
import { defineComponent, computed } from 'vue';
import { AppAgentClient, AppAgentWebsocket } from '@holochain/client';
import '@material/mwc-circular-progress';
import '@material/mwc-button';
import CreateGpgKey from './trusted/trusted/CreateGpgKey.vue';
import MyKeys from './trusted/trusted/MyKeys.vue';
import SearchKeys from './trusted/trusted/SearchKeys.vue';

export default defineComponent({
  components: {
    CreateGpgKey,
    MyKeys,
    SearchKeys
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
