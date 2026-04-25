import { ActionHash, AppClient } from "@holochain/client";
import { defineStore } from "pinia";
import { ref, watch } from "vue";
import { registerSignalHandler } from "../signals";
import { holochainClient } from "../holochain-client";

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

    const loadMyAssetSignatures = async (client: AppClient) => {
      try {
        const assetSignatures: AssetSignatureResponse[] = await client.callZome(
          {
            role_name: "checked",
            zome_name: "fetch",
            fn_name: "get_my_asset_signatures",
            payload: null,
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
      holochainClient,
      (client) => {
        if (!client) return;
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
