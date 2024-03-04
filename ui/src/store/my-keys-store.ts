import { defineStore } from "pinia";
import { ComputedRef, inject, ref, watch } from "vue";
import { GpgKeyDist, GpgKeyWithMeta } from "../trusted/trusted/types";
import { AppAgentClient } from "@holochain/client";
import { registerSignalHandler } from "../signals";

export const useMyKeysStore = defineStore("my-keys", () => {
  const loading = ref(true);
  const myKeys = ref<GpgKeyWithMeta[]>([]);

  const pushGpgKeyDist = (key: GpgKeyDist) => {
    myKeys.value.push({
      gpg_key_dist: key,
      // Assume newly created keys have a 0 reference count
      reference_count: 0,
    });
  };

  const client = inject("client") as ComputedRef<AppAgentClient>;
  const loadKeys = async (client: AppAgentClient) => {
    try {
      const r: GpgKeyWithMeta[] = await client.callZome({
        role_name: "signing_keys",
        zome_name: "signing_keys",
        fn_name: "get_my_gpg_key_dists",
        payload: null,
        cap_secret: null,
      });

      myKeys.value = [...r, ...myKeys.value];
    } catch (e) {
      // TODO Don't have the notifications store here, can I use it?
      console.error("Error loading keys", e);
    } finally {
      loading.value = false;
    }
  };

  watch(
    client,
    (client) => {
      registerSignalHandler(client, {
        myKeysStore: { pushGpgKeyDist },
      });

      loadKeys(client);
    },
    { immediate: true },
  );

  return {
    loading,
    myKeys,
    pushGpgKeyDist,
  };
});
