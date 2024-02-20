<script setup lang="ts">
import { inject, ComputedRef, ref, computed } from "vue";
import { AppAgentClient } from "@holochain/client";
import { DistributeGpgKeyRequest, GpgKeyDist } from "./types";
import { readKey } from "openpgp";
import { useNotificationsStore } from "../../store/notifications-store";
import KeyList from "../../component/KeyList.vue";

const client = inject("client") as ComputedRef<AppAgentClient>;

const notifications = useNotificationsStore();

const selected = ref<Partial<GpgKeyDist>>({});
const creating = ref(false);
const inputType = ref<'file' | 'paste'>('file');
const fileInputField = ref<HTMLElement | null>(null);
const textAreaInputField = ref<HTMLElement | null>(null);

const isGpgKeyValid = computed(() => {
  return true && selected.value.fingerprint;
});

const handleNewKeyProvided = (armoredKey: string) => {
  selected.value.public_key = armoredKey as string;

  readKey({ armoredKey: selected.value.public_key })
    .then(async (key) => {
      selected.value.fingerprint = key.getFingerprint();

      if (key.users) {
        const user = key.users[0];

        if (user.userID) {
          const userId = user.userID;

          if (userId.userID) {
            selected.value.name = userId.name;
          }
          if (userId.email) {
            selected.value.email = userId.email;
          }
        }
      }

      const expirationDate = await key.getExpirationTime();
      if (typeof expirationDate == "object") {
        selected.value.expires_at = expirationDate as Date;
      }
    })
    .catch((e) => {
      notifications.pushNotification({
        message: `${e}`,
        type: "error",
      });
    });
};

const onPublicKeySelect = async (event: Event) => {
  if (!event.target) return;

  const files = (event.target as HTMLInputElement).files;

  if (!files || files.length === 0) return;

  const file = files[0];

  const reader = new FileReader();
  reader.readAsText(file);

  reader.onload = (evt) => {
    const armoredKey = evt.target?.result;
    if (!armoredKey) return;

    handleNewKeyProvided(armoredKey as string);
  };

  reader.onerror = (evt) => {
    notifications.pushNotification({
      message: `Error reading the key file: ${evt}`,
      type: "error",
    });
  };
};

const onPublicKeyPaste = async (event: Event) => {
  if (event.type === 'paste') {
    const value = (event as ClipboardEvent).clipboardData?.getData('text/plain');
    if (!value) return;
    
    handleNewKeyProvided(value);
  } else {
    if (!event.target) return;

    const value = (event.target as HTMLTextAreaElement).value;

    // Change event will fire on click away after a paste so filter that out
    if (!value || value === selected.value.public_key) return;

    handleNewKeyProvided(value);
  }
};

const resetForm = async () => {
  selected.value.fingerprint = "";
  selected.value.expires_at = undefined;
  selected.value.public_key = "";

  if (fileInputField.value) {
    (fileInputField.value as HTMLInputElement).value = "";
  }
  if (textAreaInputField.value) {
    (textAreaInputField.value as HTMLTextAreaElement).value = "";
  }
};

const distributeGpgKey = async () => {
  if (!client.value || !selected.value.public_key) return;

  creating.value = true;

  try {
    const distributeGpgKeyRequest: DistributeGpgKeyRequest = {
      public_key: selected.value.public_key,
    };

    await client.value.callZome({
      cap_secret: null,
      role_name: "trusted",
      zome_name: "trusted",
      fn_name: "distribute_gpg_key",
      payload: distributeGpgKeyRequest,
    });

    resetForm();
  } catch (e: any) {
    notifications.pushNotification({
      message: `Error creating the gpg key: ${e}`,
      type: "error",
    });
  } finally {
    creating.value = false;
  }
};
</script>

<template>
  <p class="text-lg">Distribute your GPG public key</p>

  <div class="flex justify-center">
    <div role="tablist" class="tabs tabs-boxed w-1/2">
      <a role="tab" :class="{ tab: true, 'tab-active': inputType === 'file' }" @click="inputType = 'file'">Select file</a>
      <a role="tab" :class="{ tab: true, 'tab-active': inputType === 'paste' }" @click="inputType = 'paste'">Paste</a>
    </div>
  </div>

  <div class="flex justify-center my-3">
    <input v-if="inputType === 'file'" type="file" accept="text/*,.asc" @change="onPublicKeySelect" ref="fileInputField"
      class="file-input file-input-bordered file-input-primary" />
    <textarea v-else class="textarea textarea-ghost w-1/2" placeholder="Paste your GPG public key here"
      @change="onPublicKeyPaste" @paste="onPublicKeyPaste" ref="textAreaInputField"></textarea>
  </div>

  <div v-if="selected.fingerprint" class="mt-5">
    <p>Selected key</p>
    <!-- We know it's a partial, assume the KeyList component can handle that -->
    <KeyList :keys="[selected as GpgKeyDist]" :readonly="true"></KeyList>
  </div>

  <div class="flex justify-end my-3">
    <div class="join">
      <button class="btn btn-primary join-item" :disabled="!isGpgKeyValid || creating" @click="distributeGpgKey">
        <span v-if="creating" class="loading loading-spinner"></span>
        <span v-else>{{
          creating ? "Creating..." : "Distribute Gpg Key"
        }}</span>
      </button>
      <button class="btn btn-secondary join-item" :disabled="!selected.fingerprint" @click="resetForm">
        Cancel
      </button>
    </div>
  </div>
</template>
