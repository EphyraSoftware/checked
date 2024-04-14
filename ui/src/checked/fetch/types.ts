import {
  SignedActionHashed,
  Create,
  Update,
  Delete,
  CreateLink,
  DeleteLink,
  ActionHash,
} from "@holochain/client";

export type CheckedFetchSignal =
  | {
      type: "EntryCreated";
      action: SignedActionHashed<Create>;
      app_entry: EntryTypes;
    }
  | {
      type: "EntryUpdated";
      action: SignedActionHashed<Update>;
      app_entry: EntryTypes;
      original_app_entry: EntryTypes;
    }
  | {
      type: "EntryDeleted";
      action: SignedActionHashed<Delete>;
      original_app_entry: EntryTypes;
    }
  | {
      type: "LinkCreated";
      action: SignedActionHashed<CreateLink>;
      link_type: string;
    }
  | {
      type: "LinkDeleted";
      action: SignedActionHashed<DeleteLink>;
      link_type: string;
    };

export type EntryTypes = { type: "AssetSignature" } & AssetSignature;

export interface AssetSignature {
  fetch_url: string;
  signature: string;
  key_dist_address: ActionHash;
}
