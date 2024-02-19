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
        if (app_entry.type === "GpgKeyDist") {
          if (myKeysStore) {
            const content = JSON.parse(JSON.stringify(app_entry));
            delete content.type;
            myKeysStore.pushGpgKeyDist(content);
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
