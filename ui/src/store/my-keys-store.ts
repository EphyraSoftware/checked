import { defineStore } from "pinia";
import { ComputedRef, inject, ref, watch } from "vue";
import { GpgKeyDist, GpgKeyWithMeta } from "../trusted/trusted/types";
import { AppAgentClient } from "@holochain/client";
import { registerSignalHandler } from "../signals";

export const useMyKeysStore = defineStore("my-keys", () => {
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
    const r: GpgKeyWithMeta[] = await client.callZome({
      role_name: "trusted",
      zome_name: "trusted",
      fn_name: "get_my_gpg_key_dists",
      payload: null,
      cap_secret: null,
    });

    myKeys.value = [...r, ...myKeys.value];
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
    myKeys,
    pushGpgKeyDist,
  };
});
