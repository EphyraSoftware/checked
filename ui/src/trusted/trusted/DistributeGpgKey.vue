<script setup lang="ts">
import { inject, ComputedRef, ref, computed } from "vue";
import { AppAgentClient, Record } from "@holochain/client";
import { DistributeGpgKeyRequest, GpgKeyDist } from "./types";
import { readKey } from "openpgp";
import { useMyKeysStore } from "../../store/my-keys-store";
import { useNotificationsStore } from "../../store/notifications-store";
import KeyList from "../../component/KeyList.vue";

const emit = defineEmits<{
  (e: "gpg-key-dist-created", hash: Uint8Array): void;
}>();

const selected = ref<Partial<GpgKeyDist>>({});
const creating = ref(false);
const inputField = ref<HTMLElement | null>(null);

const client = inject("client") as ComputedRef<AppAgentClient>;

const notifications = useNotificationsStore();
const myKeysStore = useMyKeysStore();

const isGpgKeyValid = computed(() => {
  return true && selected.value.fingerprint;
});

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

  reader.onerror = (evt) => {
    notifications.pushNotification({
      message: `Error reading the key file: ${evt}`,
      type: "error",
    });
  };
};

const resetForm = async () => {
  selected.value.fingerprint = "";
  selected.value.expires_at = undefined;
  selected.value.public_key = "";

  if (inputField.value) {
    (inputField.value as HTMLInputElement).value = "";
  }
};

const distributeGpgKey = async () => {
  if (!client.value || !selected.value.public_key) return;

  creating.value = true;

  try {
    const distributeGpgKeyRequest: DistributeGpgKeyRequest = {
      public_key: selected.value.public_key,
    };

    const record: Record = await client.value.callZome({
      cap_secret: null,
      role_name: "trusted",
      zome_name: "trusted",
      fn_name: "distribute_gpg_key",
      payload: distributeGpgKeyRequest,
    });
    emit("gpg-key-dist-created", record.signed_action.hashed.hash);

    myKeysStore.insertRecord(record);

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
  <p class="text-lg">Distribute GPG public key</p>

  <div class="flex justify-center my-3">
    <input
      type="file"
      accept="text/*,.asc"
      @change="onPublicKeySelect"
      ref="inputField"
      class="file-input file-input-bordered file-input-primary"
    />
  </div>

  <div v-if="selected.fingerprint" class="mt-5">
    <p>Selected key</p>
    <KeyList :keys="[selected]" :readonly="true"></KeyList>
  </div>

  <div class="flex justify-center my-3">
    <div class="join">
      <button
        class="btn btn-primary join-item"
        :disabled="!isGpgKeyValid || creating"
        @click="distributeGpgKey"
      >
        <span v-if="creating" class="loading loading-spinner"></span>
        <span v-else>{{
          creating ? "Creating..." : "Distribute Gpg Key"
        }}</span>
      </button>
      <button
        class="btn btn-secondary join-item"
        :disabled="!selected.fingerprint"
        @click="resetForm"
      >
        Cancel
      </button>
    </div>
  </div>
</template>
