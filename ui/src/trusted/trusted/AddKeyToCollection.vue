<script setup lang="ts">
import { GpgKeyDist } from "./types";
import KeyList from "../../component/KeyList.vue";
import { useKeyCollectionsStore } from "../../store/key-collections-store";
import { ComputedRef, inject, ref, watch } from "vue";
import CreateKeyCollection from "./CreateKeyCollection.vue";
import { useNotificationsStore } from "../../store/notifications-store";
import { AppAgentClient } from "@holochain/client";

const props = defineProps<{
  selectedKey: GpgKeyDist;
}>();

const emit = defineEmits<{
  (e: "added"): void;
}>();

const client = inject("client") as ComputedRef<AppAgentClient>;

const keyCollectionsStore = useKeyCollectionsStore();
const notificationsStore = useNotificationsStore();

const selectedCollection = ref<string>("");
const expectCreated = ref<string | null>(null);
const linking = ref(false);

watch(
  keyCollectionsStore.keyCollections,
  (newVal) => {
    if (newVal.length > 0) {
      if (
        expectCreated.value &&
        newVal.map((c) => c.name).indexOf(expectCreated.value) !== -1
      ) {
        selectedCollection.value = expectCreated.value;
        expectCreated.value = null;
      } else {
        selectedCollection.value = newVal[0].name;
      }
    }
  },
  { immediate: true },
);

const onKeyCollectionCreated = (name: string) => {
  expectCreated.value = name;

  // To reset the UI if something goes wrong, 5s might be a bit aggressive but if the signal shows up then the select will show the new collection anyway.
  setTimeout(() => {
    // If we haven't started creating a new collection, then we should clear this field
    if (expectCreated.value === name) {
      expectCreated.value = null;
      notificationsStore.pushNotification({
        message: `New key collection '${name}' may not have been created`,
        type: "warning",
      });
    }
  }, 5000);
};

const addKeyToCollection = () => {
  if (!selectedCollection.value || !client.value || linking.value) return;

  linking.value = true;

  try {
    client.value.callZome({
      cap_secret: null,
      role_name: "trusted",
      zome_name: "trusted",
      fn_name: "link_gpg_key_to_key_collection",
      payload: {
        gpg_key_fingerprint: props.selectedKey.fingerprint,
        key_collection_name: selectedCollection.value,
      },
    });

    keyCollectionsStore.addKeyToCollection(
      selectedCollection.value,
      props.selectedKey,
    );

    emit("added");

    notificationsStore.pushNotification({
      message: `Key added to collection '${selectedCollection.value}'`,
      type: "info",
    });
  } catch (e: any) {
    notificationsStore.pushNotification({
      message: `Error adding key to collection - ${e}`,
      type: "error",
      timeout: 5000,
    });
  }
  {
    linking.value = false;
  }
};
</script>

<template>
  <KeyList :keys="[selectedKey]" :readonly="true"></KeyList>

  <p class="mt-5">Pick a key collection</p>
  <div class="join w-full">
    <select
      v-model="selectedCollection"
      class="select select-bordered w-full join-item"
    >
      <option disabled value="">Select a key collection</option>
      <option
        v-for="c in keyCollectionsStore.keyCollections"
        v-bind:key="c.name"
        :value="c.name"
      >
        {{ c.name }}
      </option>
    </select>
    <button
      class="btn btn-primary join-item"
      :disabled="!selectedCollection"
      @click="addKeyToCollection"
    >
      Add to collection
    </button>
  </div>

  <p class="mt-5">Or create a new one</p>
  <div v-if="expectCreated" class="flex grow justify-center">
    <span class="loading loading-spinner"></span>
  </div>
  <div v-else>
    <CreateKeyCollection
      @created="onKeyCollectionCreated"
    ></CreateKeyCollection>
  </div>
</template>
