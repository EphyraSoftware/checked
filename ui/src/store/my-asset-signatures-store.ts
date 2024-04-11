import { ActionHash, AppAgentClient } from "@holochain/client";
import { defineStore } from "pinia";
import { ComputedRef, inject, ref, watch } from "vue";
import { registerSignalHandler } from "../signals";

export interface AssetSignatureResponse {
  fetch_url: string;
  signature: string;
  key_dist_address: ActionHash;
  created_at: number;
}

export const useMyAssetSignaturesStore = defineStore(
  "my-asset-signatures",
  () => {
    const loading = ref(true);
    const myAssetSignatures = ref<AssetSignatureResponse[]>([]);

    const pushAssetSignature = (assetSignature: AssetSignatureResponse) => {
      myAssetSignatures.value.push(assetSignature);
    };

    const client = inject("client") as ComputedRef<AppAgentClient>;

    const loadMyAssetSignatures = async (client: AppAgentClient) => {
      try {
        const assetSignatures: AssetSignatureResponse[] = await client.callZome(
          {
            role_name: "checked",
            zome_name: "fetch",
            fn_name: "get_my_asset_signatures",
            payload: null,
            cap_secret: null,
          },
        );

        myAssetSignatures.value = [
          ...assetSignatures,
          ...myAssetSignatures.value,
        ];
      } catch (e) {
        // TODO Don't have the notifications store here, can I use it?
        console.error("Error loading my asset signatures", e);
      } finally {
        loading.value = false;
      }
    };

    watch(
      client,
      (client) => {
        registerSignalHandler(client, {
          myAssetSignaturesStore: { pushAssetSignature },
        });

        loadMyAssetSignatures(client);
      },
      { immediate: true },
    );

    return {
      loading,
      myAssetSignatures,
      pushAssetSignature,
    };
  },
);
