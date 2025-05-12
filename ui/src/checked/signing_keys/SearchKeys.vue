<script setup lang="ts">
import { AppClient } from "@holochain/client";
import { ComputedRef, inject, ref } from "vue";
import { SearchKeysRequest, VfKeyResponse } from "./types";
import { useNotificationsStore } from "../../store/notifications-store";
import KeyList from "../../component/KeyList.vue";
import AddKeyToCollection from "./AddKeyToCollection.vue";
import IconButton from "../../component/IconButton.vue";

const searchQuery = ref("");
const searching = ref(false);
const results = ref<VfKeyResponse[]>([]);
const selectedKeyForAdd = ref<VfKeyResponse | null>(null);

const client = inject("client") as ComputedRef<AppClient>;

const notifications = useNotificationsStore();

const searchKeys = async () => {
  if (!client.value || searching.value) return;

  searching.value = true;

  try {
    const request: SearchKeysRequest = {
      agent_pub_key: searchQuery.value,
    };

    results.value = await client.value.callZome({
      role_name: "checked",
      zome_name: "signing_keys",
      fn_name: "search_keys",
      payload: request,
    });
  } catch (e) {
    notifications.pushNotification({
      message: `Error searching for keys - ${e}`,
      type: "error",
      timeout: 5000,
    });
  } finally {
    setTimeout(() => {
      searching.value = false;
    }, 500);
  }
};

const onAddKey = (key: VfKeyResponse) => {
  selectedKeyForAdd.value = key;
};

const onKeyAdded = () => {
  selectedKeyForAdd.value = null;
};

// Not delightful, a competent base64 library that works in the browser rather than node is needed :)
function bytesToBase64(bytes: Uint8Array) {
  const binString = Array.from(bytes, (byte) =>
    String.fromCodePoint(byte),
  ).join("");
  return btoa(binString).replace(/\+/g, "-").replace(/\//g, "_");
}

const onCopyAgentKey = () => {
  if (!client.value) return;

  const agentKeyText = `u${bytesToBase64(client.value.myPubKey)}`;
  navigator.clipboard.writeText(agentKeyText);

  notifications.pushNotification({
    message: "Your agent key has been copied to the clipboard",
    type: "info",
    timeout: 5000,
  });
};
</script>

<template>
  <template v-if="!selectedKeyForAdd">
    <div class="flex flex-row">
      <div class="flex-grow">
        <p class="pb-3 text-lg font-bold">
          Search keys distributed by another agent
        </p>
        <p class="text-sm italic pb-3">
          You'll need somebody to send you their agent key to use here. You can
          copy yours from this screen to send it to other people.
        </p>
      </div>
      <div>
        <IconButton
          title="Copy my agent key"
          icon="fa-regular fa-clipboard"
          @click="onCopyAgentKey"
        />
      </div>
    </div>

    <form @submit="(e) => e.preventDefault()">
      <div class="join flex w-full">
        <input
          type="text"
          class="input input-bordered join-item grow"
          placeholder="Agent key"
          name="search-for-keys"
          id="search-for-keys"
          v-model="searchQuery"
        />
        <button
          class="btn btn-primary join-item min-w-24"
          @click="searchKeys"
          :disabled="!searchQuery"
        >
          <span v-if="searching" class="loading loading-spinner"></span>
          <span v-else>Search</span>
        </button>
      </div>
    </form>

    <div class="mt-5">
      <p class="text-lg font-bold">Search results</p>
      <KeyList
        v-if="results.length"
        :key-dist-list="results"
        :readonly="false"
        @add-key="onAddKey"
      ></KeyList>
      <div v-else class="w-full flex justify-center">
        <p class="text-sm italic">No results</p>
      </div>
    </div>
  </template>
  <template v-else>
    <div class="flex">
      <div class="grow">
        <p class="text-lg">Add key to a collection?</p>
        <p>You should only add keys you trust!</p>
      </div>
      <div>
        <button class="btn btn-accent" @click="selectedKeyForAdd = null">
          Cancel
        </button>
      </div>
    </div>

    <AddKeyToCollection
      :selected-key="selectedKeyForAdd"
      @added="onKeyAdded"
    ></AddKeyToCollection>
  </template>
</template>
