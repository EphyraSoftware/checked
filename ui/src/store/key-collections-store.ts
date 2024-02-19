import { AppAgentClient, Record } from "@holochain/client";
import { decode } from "@msgpack/msgpack";
import { defineStore } from "pinia";
import { ComputedRef, inject, ref, watch } from "vue";
import { KeyCollection } from "../trusted/trusted/types";
import { registerSignalHandler } from "../signals";

export interface StoreKeyCollection {
  name: string;
  keys: object[]; // TODO type
}

type KeyCollections = { [name: string]: StoreKeyCollection };

export const useKeyCollectionsStore = defineStore("key-collections", () => {
  const keyCollections = ref<KeyCollections>({});

  const pushKeyCollection = (collection: StoreKeyCollection) => {
    keyCollections.value[collection.name] = collection;
  };

  const client = inject("client") as ComputedRef<AppAgentClient>;
  const loadKeyCollections = async (client: AppAgentClient) => {
    console.log("loading key collections");

    const r: Record[] = await client.callZome({
      role_name: "trusted",
      zome_name: "trusted",
      fn_name: "get_my_key_collections",
      payload: null,
      cap_secret: null,
    });

    keyCollections.value = {
      ...r.reduce((acc, record) => {
        const keyCollection = decode(
          (record.entry as any).Present.entry,
        ) as KeyCollection;
        acc[keyCollection.name] = keyCollection;
        return acc;
      }, {} as KeyCollections),
      ...keyCollections.value,
    };
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
  };
});
