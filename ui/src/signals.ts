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
          if (!myKeysStore) {
            console.error(
              "App configuration error: Got a signal for a GPG key distribution but no pushGpgKeyDist function was provided",
            );
            return;
          }

          delete app_entry.type;
          myKeysStore.pushGpgKeyDist(app_entry);
        } else if (app_entry.type === "KeyCollection") {
          if (!keyCollectionsStore) {
            console.error(
              "App configuration error: Got a signal for a key collection but no pushKeyCollection function was provided",
            );
            return;
          }

          delete app_entry.type;
          app_entry["keys"] = [];
          keyCollectionsStore.pushKeyCollection(app_entry);
        }
      }
    }
  });
};
