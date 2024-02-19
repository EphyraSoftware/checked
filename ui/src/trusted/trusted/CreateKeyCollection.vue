<script setup lang="ts">
import { AppAgentClient } from "@holochain/client";
import { ComputedRef, inject, ref } from "vue";
import { useNotificationsStore } from "../../store/notifications-store";

const client = inject("client") as ComputedRef<AppAgentClient>;

const notifications = useNotificationsStore();

const keyCollectionName = ref("");
const creating = ref(false);

const createKeyCollection = async () => {
  if (creating.value || !client.value) return;

  creating.value = true;

  try {
    client.value.callZome({
      cap_secret: null,
      role_name: "trusted",
      zome_name: "trusted",
      fn_name: "create_key_collection",
      payload: {
        name: keyCollectionName.value,
      },
    });

    keyCollectionName.value = "";
  } catch (e: any) {
    notifications.pushNotification({
      message: `Error creating key collection - ${e}`,
      type: "error",
      timeout: 5000,
    });
  } finally {
    creating.value = false;
  }
};
</script>

<template>
  <div>
    <h1>Create Key Collection</h1>
  </div>
  <form @submit="(e) => e.preventDefault()">
    <div class="join flex w-full">
      <input
        type="text"
        class="input input-bordered join-item grow"
        placeholder="Name"
        minlength="3"
        name="key-collection-name"
        id="key-collection-name"
        v-model="keyCollectionName"
      />
      <button
        class="btn btn-primary join-item min-w-24"
        @click="createKeyCollection"
        :disabled="!keyCollectionName || keyCollectionName.length < 3"
      >
        <span v-if="creating" class="loading loading-spinner"></span>
        <span v-else>Create</span>
      </button>
    </div>
  </form>
</template>
