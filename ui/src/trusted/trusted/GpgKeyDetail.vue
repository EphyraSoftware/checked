<template>
  <div v-if="!loading">
    <div v-if="record" style="display: flex; flex-direction: column">
      <div style="display: flex; flex-direction: row">
        <span style="flex: 1"></span>
      
      </div>

      <div style="display: flex; flex-direction: row; margin-bottom: 16px;">
	<span style="margin-right: 4px"><strong>Fingerprint: </strong></span>
 	<span style="white-space: pre-line">{{  gpgKey?.fingerprint }} </span>
      </div>

    </div>
    
    <span v-else>The requested gpg key was not found.</span>
  </div>

  <div v-else style="display: flex; flex: 1; align-items: center; justify-content: center">
    <mwc-circular-progress indeterminate></mwc-circular-progress>
  </div>

</template>

<script lang="ts">
import { defineComponent, inject, ComputedRef } from 'vue';
import { decode } from '@msgpack/msgpack';
import { AppAgentClient, Record, AgentPubKey, EntryHash, ActionHash, DnaHash } from '@holochain/client';
import { GpgKeyDist } from './types';
import '@material/mwc-circular-progress';
import '@material/mwc-icon-button';
import '@material/mwc-snackbar';
import { Snackbar } from '@material/mwc-snackbar';

export default defineComponent({
  props: {
    gpgKeyHash: {
      type: Object,
      required: true
    }
  },
  data(): { record: Record | undefined; loading: boolean;  } {
    return {
      record: undefined,
      loading: true,
    }
  },
  computed: {
    gpgKey() {
      if (!this.record) return undefined;
      return decode((this.record.entry as any).Present.entry) as GpgKeyDist;
    }
  },
  async mounted() {
    if (this.gpgKeyHash === undefined) {
      throw new Error(`The gpgKeyHash input is required for the GpgKeyDetail element`);
    }

    await this.fetchGpgKey();
  },
  methods: {
    async fetchGpgKey() {
      this.loading = true;
      this.record = undefined;

      this.record = await this.client.callZome({
        cap_secret: null,
        role_name: 'trusted',
        zome_name: 'trusted',
        fn_name: 'get_gpg_key_dist',
        payload: this.gpgKeyHash,
      });

      this.loading = false;
    },
  },
  emits: ['gpg-key-deleted'],
  setup() {
    const client = (inject('client') as ComputedRef<AppAgentClient>).value;
    return {
      client
    };
  },
})
</script>
