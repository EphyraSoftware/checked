<script setup lang="ts">
import { AppAgentClient } from "@holochain/client";
import { ComputedRef, inject, ref } from "vue";
import { SearchKeysRequest, VfKeyResponse } from "./types";
import { useNotificationsStore } from "../../store/notifications-store";
import KeyList from "../../component/KeyList.vue";
import AddKeyToCollection from "./AddKeyToCollection.vue";

const searchQuery = ref("");
const searching = ref(false);
const results = ref<VfKeyResponse[]>([]);
const selectedKeyForAdd = ref<VfKeyResponse | null>(null);

const client = inject("client") as ComputedRef<AppAgentClient>;

const notifications = useNotificationsStore();

const searchKeys = async () => {
  if (!client.value || searching.value) return;

  searching.value = true;

  try {
    const request: SearchKeysRequest = {
      query: searchQuery.value,
    };

    results.value = await client.value.callZome({
      role_name: "checked",
      zome_name: "signing_keys",
      fn_name: "search_keys",
      payload: request,
      cap_secret: null,
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
</script>

<template>
  <template v-if="!selectedKeyForAdd">
    <p>Search for keys by user id, email or fingerprint.</p>
    <p class="text-sm italic">
      An exact match is required for all fields and the fingerprint should be in
      upper-case hex.
    </p>

    <form @submit="(e) => e.preventDefault()">
      <div class="join flex w-full">
        <input
          type="text"
          class="input input-bordered join-item grow"
          placeholder="Search"
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
      <p>Search results</p>
      <KeyList
        :key-dist-list="results"
        :readonly="false"
        @add-key="onAddKey"
      ></KeyList>
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
