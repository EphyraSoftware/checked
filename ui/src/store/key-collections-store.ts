import { AppAgentClient } from "@holochain/client";
import { defineStore } from "pinia";
import { ComputedRef, inject, ref, watch } from "vue";
import { GpgKeyWithMeta } from "../trusted/trusted/types";
import { registerSignalHandler } from "../signals";

export interface KeyCollectionWithKeys {
  name: string;
  gpg_keys: GpgKeyWithMeta[];
}

export const useKeyCollectionsStore = defineStore("key-collections", () => {
  const keyCollections = ref<KeyCollectionWithKeys[]>([]);

  const pushKeyCollection = (collection: KeyCollectionWithKeys) => {
    keyCollections.value.push(collection);
  };

  const addKeyToCollection = (name: string, key: GpgKeyWithMeta) => {
    const existingCollection = keyCollections.value.find(
      (c) => c.name === name,
    );
    if (existingCollection) {
      existingCollection.gpg_keys.push(key);
    }
  };

  const client = inject("client") as ComputedRef<AppAgentClient>;
  const loadKeyCollections = async (client: AppAgentClient) => {
    const collections: KeyCollectionWithKeys[] = await client.callZome({
      role_name: "trusted",
      zome_name: "trusted",
      fn_name: "get_my_key_collections",
      payload: null,
      cap_secret: null,
    });

    keyCollections.value = [...collections, ...keyCollections.value];
  };

  watch(
    client,
    (client) => {
      registerSignalHandler(client, {
        keyCollectionsStore: { pushKeyCollection },
      });

      loadKeyCollections(client);
    },
    { immediate: true },
  );

  return {
    keyCollections,
    pushKeyCollection,
    addKeyToCollection,
  };
});
