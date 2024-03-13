import {
  ActionHash,
  AppAgentClient,
  SignedActionHashed,
} from "@holochain/client";
import { KeyCollectionWithKeys } from "./store/key-collections-store";
import { VerificationKeyDist } from "./trusted/trusted/types";

export const registerSignalHandler = (
  client: AppAgentClient,
  {
    myKeysStore,
    keyCollectionsStore,
  }: Partial<{
    myKeysStore: {
      pushVfKeyDist: (
        keyDist: VerificationKeyDist,
        keyDistAddress: ActionHash,
      ) => void;
    };
    keyCollectionsStore: {
      pushKeyCollection: (collection: KeyCollectionWithKeys) => void;
    };
  }>,
) => {
  client.on("signal", (signal) => {
    // TODO very messy type work, improve me!
    if (signal.zome_name === "signing_keys") {
      if ((signal.payload as any).type === "EntryCreated") {
        const app_entry = (signal.payload as any).app_entry;
        const action = (signal.payload as any).action as SignedActionHashed;
        if (app_entry.type === "VerificationKeyDist") {
          if (myKeysStore) {
            const content = JSON.parse(JSON.stringify(app_entry));
            delete content.type;
            myKeysStore.pushVfKeyDist(content, action.hashed.hash);
          }
        } else if (app_entry.type === "KeyCollection") {
          if (keyCollectionsStore) {
            const content = JSON.parse(JSON.stringify(app_entry));
            delete content.type;
            content["gpg_keys"] = [];
            keyCollectionsStore.pushKeyCollection(content);
          }
        } else {
          console.warn("Unknown signal type", signal);
        }
      }
    }
  });
};
