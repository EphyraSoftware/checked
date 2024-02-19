import { AppAgentClient } from "@holochain/client";
import { KeyCollectionWithKeys } from "./store/key-collections-store";
import { GpgKeyDist } from "./trusted/trusted/types";

export const registerSignalHandler = (
  client: AppAgentClient,
  {
    myKeysStore,
    keyCollectionsStore,
  }: Partial<{
    myKeysStore: {
      pushGpgKeyDist: (key: GpgKeyDist) => void;
    };
    keyCollectionsStore: {
      pushKeyCollection: (collection: KeyCollectionWithKeys) => void;
    };
  }>,
) => {
  client.on("signal", (signal) => {
    // TODO very messy type work, improve me!
    if (signal.zome_name === "trusted") {
      if ((signal.payload as any).type === "EntryCreated") {
        const app_entry = (signal.payload as any).app_entry;
        if (app_entry.type === "GpgKeyDist" && myKeysStore) {
          delete app_entry.type;
          myKeysStore.pushGpgKeyDist(app_entry);
        } else if (app_entry.type === "KeyCollection" && keyCollectionsStore) {
          delete app_entry.type;
          app_entry["gpg_keys"] = [];
          keyCollectionsStore.pushKeyCollection(app_entry);
        } else {
            console.warn("Unknown signal type", signal);
        }
      }
    }
  });
};
