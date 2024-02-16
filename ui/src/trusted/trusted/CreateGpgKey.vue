<template>
  <mwc-snackbar ref="create-error"></mwc-snackbar>

  <div style="display: flex; flex-direction: column">
    <span style="font-size: 18px">Create Gpg Key</span>
  
    <div style="margin-bottom: 16px">
      <mwc-textarea outlined label="Fingerprint" :value="fingerprint" @input="fingerprint = $event.target.value" required></mwc-textarea>
    </div>

  
    <mwc-button 
      raised
      label="Create Gpg Key"
      :disabled="!isGpgKeyValid"
      @click="createGpgKey"
    ></mwc-button>
  </div>
</template>
<script lang="ts">
import { defineComponent, inject, ComputedRef } from 'vue';
import { AppAgentClient, Record, AgentPubKey, EntryHash, ActionHash, DnaHash } from '@holochain/client';
import { GpgKey } from './types';
import '@material/mwc-button';
import '@material/mwc-icon-button';
import '@material/mwc-snackbar';
import { Snackbar } from '@material/mwc-snackbar';
import '@material/mwc-textarea';

export default defineComponent({
  data(): {
    fingerprint: string;
  } {
    return { 
      fingerprint: '',
    }
  },

  props: {    publicKey: {
      type: null,
      required: true
    },
  },
  computed: {
    isGpgKeyValid() {
    return true && this.fingerprint !== '';
    },
  },
  mounted() {
    if (this.publicKey === undefined) {
      throw new Error(`The publicKey input is required for the CreateGpgKey element`);
    }
  },
  methods: {
    async createGpgKey() {
      const gpgKey: GpgKey = { 
        public_key: this.publicKey as Array<number>,

        fingerprint: this.fingerprint!,
      };

      try {
        const record: Record = await this.client.callZome({
          cap_secret: null,
          role_name: 'trusted',
          zome_name: 'trusted',
          fn_name: 'create_gpg_key',
          payload: gpgKey,
        });
        this.$emit('gpg-key-created', record.signed_action.hashed.hash);
      } catch (e: any) {
        const errorSnackbar = this.$refs['create-error'] as Snackbar;
        errorSnackbar.labelText = `Error creating the gpg key: ${e.data}`;
        errorSnackbar.show();
      }
    },
  },
  emits: ['gpg-key-created'],
  setup() {
    const client = (inject('client') as ComputedRef<AppAgentClient>).value;
    return {
      client,
    };
  },
})
</script>
