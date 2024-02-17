<template>
  <mwc-snackbar ref="create-error"></mwc-snackbar>

  <div style="display: flex; flex-direction: column">
    <span style="font-size: 18px">Distribute GPG public key</span>

    <input type="file" accept="text/*,.asc" @change="onPublicKeySelect" ref="inputField" />

    <template v-if="fingerprint">
      <p>Selected key has fingerprint: <span style="font-style: italic;">{{ fingerprint }}</span></p>
      <p>Selected key expires on: <span style="font-style: italic;">{{ expirationDate }}</span></p>
    </template>

    <mwc-button raised :label="creating ? 'Creating...' : 'Distribute Gpg Key'" :disabled="!isGpgKeyValid || creating" @click="distributeGpgKey"></mwc-button>
  </div>
</template>
<script lang="ts">
import { defineComponent, inject, ComputedRef } from 'vue';
import { AppAgentClient, Record, AgentPubKey, EntryHash, ActionHash, DnaHash } from '@holochain/client';
import { DistributeGpgKeyRequest } from './types';
import '@material/mwc-button';
import '@material/mwc-icon-button';
import '@material/mwc-snackbar';
import { Snackbar } from '@material/mwc-snackbar';
import '@material/mwc-textarea';
import { readKey } from 'openpgp'
import { useMyKeysStore } from '../../store/my-keys-store';

export default defineComponent({
  data(): {
    fingerprint: string;
    expirationDate: Date | null;
    selectedKey: string;
    creating: boolean;
    myKeysStore: any;
  } {
    return {
      fingerprint: '',
      expirationDate: null,
      selectedKey: '',
      creating: false,
      myKeysStore: useMyKeysStore(),
    }
  },
  computed: {
    isGpgKeyValid() {
      return true && this.fingerprint !== '';
    },
  },
  mounted() {
  },
  methods: {
    async onPublicKeySelect(event: Event) {
      if (!event.target) return;

      const files = (event.target as HTMLInputElement).files

      if (!files || files.length === 0) return;

      const file = files[0];

      const reader = new FileReader();
      reader.readAsText(file);

      reader.onload = (evt) => {
        const armoredKey = evt.target?.result;

        if (!armoredKey) return;

        this.selectedKey = armoredKey as string;

        readKey({ armoredKey: this.selectedKey }).then(async (key) => {
          this.fingerprint = key.getFingerprint();

          const expirationDate = await key.getExpirationTime();
          if (typeof expirationDate == 'object') {
            this.expirationDate = expirationDate;
          }
        }).catch((e) => {
          const errorSnackbar = this.$refs['create-error'] as Snackbar;
          errorSnackbar.labelText = `${e}`;
          errorSnackbar.show();
        });
      }

      reader.onerror = (evt) => {
        const errorSnackbar = this.$refs['create-error'] as Snackbar;
        errorSnackbar.labelText = `Error reading the key file: ${evt}`;
        errorSnackbar.show();
      }
    },
    async distributeGpgKey() {
      this.creating = true;

      try {
        const distributeGpgKeyRequest: DistributeGpgKeyRequest = {
          public_key: this.selectedKey,
        };

        const record: Record = await this.client.callZome({
          cap_secret: null,
          role_name: 'trusted',
          zome_name: 'trusted',
          fn_name: 'distribute_gpg_key',
          payload: distributeGpgKeyRequest,
        });
        this.$emit('gpg-key-dist-created', record.signed_action.hashed.hash);

        this.myKeysStore.insertRecord(record);

        this.fingerprint = '';
        this.expirationDate = null;
        this.selectedKey = '';

        const inputField = this.$refs['inputField'] as HTMLInputElement | null;
        if (inputField) {
          inputField.value = '';
        }
      } catch (e: any) {
        const errorSnackbar = this.$refs['create-error'] as Snackbar;
        errorSnackbar.labelText = `Error creating the gpg key: ${e}`;
        errorSnackbar.show();
      } finally {
        this.creating = false;
      }
    },
  },
  emits: ['gpg-key-dist-created'],
  setup() {
    const client = (inject('client') as ComputedRef<AppAgentClient>).value;
    return {
      client,
    };
  },
})
</script>
