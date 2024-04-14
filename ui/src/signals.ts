import { AppAgentClient } from "@holochain/client";
import { KeyCollectionWithKeys } from "./store/key-collections-store";
import {
  CheckedSigningKeysSignal,
  VfKeyResponse,
} from "./checked/signing_keys/types";
import { AssetSignatureResponse } from "./store/my-asset-signatures-store";
import { CheckedFetchSignal } from "./checked/fetch/types";

export const registerSignalHandler = (
  client: AppAgentClient,
  {
    myKeysStore,
    keyCollectionsStore,
    myAssetSignaturesStore,
  }: Partial<{
    myKeysStore: {
      pushVfKeyDist: (vfKey: VfKeyResponse) => void;
    };
    keyCollectionsStore: {
      pushKeyCollection: (collection: KeyCollectionWithKeys) => void;
    };
    myAssetSignaturesStore: {
      pushAssetSignature: (assetSignature: AssetSignatureResponse) => void;
    };
  }>,
) => {
  client.on("signal", (signal) => {
    if (signal.zome_name === "signing_keys") {
      const payload = signal.payload as CheckedSigningKeysSignal;

      if (payload.type === "EntryCreated") {
        const app_entry = payload.app_entry;
        const action = payload.action;

        if (app_entry.type === "VerificationKeyDist") {
          if (myKeysStore) {
            myKeysStore.pushVfKeyDist({
              verification_key_dist: {
                verification_key: app_entry.verification_key,
                key_type: app_entry.key_type,
                name: app_entry.name,
                expires_at: app_entry.expires_at,
                marks: [], // Expect no marks initially
              },
              key_dist_address: action.hashed.hash,
              created_at: action.hashed.content.timestamp,
              reference_count: 0, // Assume newly created keys have a 0 reference count
            }); //(content, action.hashed.hash);
          }
        } else if (app_entry.type === "KeyCollection") {
          if (keyCollectionsStore) {
            keyCollectionsStore.pushKeyCollection({
              name: app_entry.name,
              keys: [],
            });
          }
        } else {
          console.warn("Unknown app entry type in signal", signal);
        }
      }
    } else if (signal.zome_name === "fetch") {
      const payload = signal.payload as CheckedFetchSignal;

      if (payload.type === "EntryCreated") {
        const app_entry = payload.app_entry;
        const action = payload.action;

        if (app_entry.type === "AssetSignature") {
          if (myAssetSignaturesStore) {
            myAssetSignaturesStore.pushAssetSignature({
              fetch_url: app_entry.fetch_url,
              signature: app_entry.signature,
              key_dist_address: app_entry.key_dist_address,
              created_at: action.hashed.content.timestamp,
            });
          }
        } else {
          console.warn("Unknown app entry type in signal", signal);
        }
      }
    }
  });
};
