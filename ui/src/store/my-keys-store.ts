import { defineStore } from "pinia";
import { ComputedRef, inject, ref, watch } from "vue";
import {
  VerificationKeyDist,
  VfKeyResponse,
} from "../checked/signing_keys/types";
import { ActionHash, AppAgentClient } from "@holochain/client";
import { registerSignalHandler } from "../signals";

export const useMyKeysStore = defineStore("my-keys", () => {
  const loading = ref(true);
  const myKeys = ref<VfKeyResponse[]>([]);

  const pushVfKeyDist = (
    keyDist: VerificationKeyDist,
    keyDistAddress: ActionHash,
  ) => {
    myKeys.value.push({
      verification_key_dist: {
        verification_key: keyDist.verification_key,
        key_type: keyDist.key_type,
        name: keyDist.name,
        expires_at: keyDist.expires_at,
        marks: [], // Should have no marks initially
      },
      key_dist_address: keyDistAddress,
      created_at: Date.now(), // Approximate creation time, will correct on next load
      reference_count: 0, // Assume newly created keys have a 0 reference count
    });
  };

  const client = inject("client") as ComputedRef<AppAgentClient>;
  const loadKeys = async (client: AppAgentClient) => {
    try {
      const r: VfKeyResponse[] = await client.callZome({
        role_name: "checked",
        zome_name: "signing_keys",
        fn_name: "get_my_verification_key_distributions",
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
        myKeysStore: { pushVfKeyDist },
      });

      loadKeys(client).catch((e) => {
        console.error("Error loading keys", e);
      });
    },
    { immediate: true },
  );

  return {
    loading,
    myKeys,
    pushVfKeyDist,
  };
});
