import { defineStore } from "pinia";
import { ComputedRef, inject, ref, watch } from "vue";
import { GpgKeyDist } from "../trusted/trusted/types";
import { AppAgentClient, Record } from "@holochain/client";
import { decode } from "@msgpack/msgpack";
import { registerSignalHandler } from "../signals";

export const useMyKeysStore = defineStore("my-keys", () => {
  const myKeys = ref<GpgKeyDist[]>([]);

  const pushGpgKeyDist = (key: GpgKeyDist) => {
    myKeys.value.push(key);
  };

  const client = inject("client") as ComputedRef<AppAgentClient>;
  const loadKeys = async (client: AppAgentClient) => {
    const r: Record[] = await client.callZome({
      role_name: "trusted",
      zome_name: "trusted",
      fn_name: "get_my_keys",
      payload: null,
      cap_secret: null,
    });

    myKeys.value = [
      ...r.map((record) => {
        return decode((record.entry as any).Present.entry) as GpgKeyDist;
      }),
      ...myKeys.value,
    ];
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
