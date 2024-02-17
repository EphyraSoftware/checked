<script setup lang="ts">
import { AppAgentClient, Record } from "@holochain/client";
import { ComputedRef, inject, ref } from "vue";
import { GpgKeyDist, SearchKeysRequest } from "./types";
import { decode } from "@msgpack/msgpack";
import { useNotificationsStore } from "../../store/notifications-store";
import KeyList from "../../component/KeyList.vue";

const searchQuery = ref("");
const searching = ref(false);
const results = ref<GpgKeyDist[]>([]);

const client = inject("client") as ComputedRef<AppAgentClient>;

const notifications = useNotificationsStore();

const searchKeys = async () => {
  if (!client.value || searching.value) return;

  searching.value = true;

  try {
    const request: SearchKeysRequest = {
      query: searchQuery.value,
    };

    const r: Record[] = await client.value.callZome({
      role_name: "trusted",
      zome_name: "trusted",
      fn_name: "search_keys",
      payload: request,
      cap_secret: null,
    });

    results.value = r.map(
      (record) => decode((record.entry as any).Present.entry) as GpgKeyDist,
    );
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
</script>

<template>
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
    <KeyList :keys="results"></KeyList>
  </div>
</template>
