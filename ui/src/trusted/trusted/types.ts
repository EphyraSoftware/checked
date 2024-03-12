import {
  SignedActionHashed,
  Create,
  Update,
  Delete,
  CreateLink,
  DeleteLink,
} from "@holochain/client";

export type TrustedSignal =
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

export type EntryTypes = { type: "GpgKeyDist" } & GpgKeyDist;

export interface DistributeGpgKeyRequest {
  name: string;
  verification_key: string;
  key_type: 'MiniSignEd25519',
  proof: string;
  proof_signature: string;
}

export interface VerificationKeyDist {
  verification_key: string;
  key_type: {'MiniSignEd25519': null};
  proof: string;
  proof_signature: number[];
  name: string;
  expires_at?: Date;
}

export interface GpgKeyDist {
  public_key: string;
  fingerprint: string;
  name: string;
  email?: string;
  expires_at?: Date;
}

export interface GpgKeyWithMeta {
  gpg_key_dist: GpgKeyDist;
  reference_count: number;
}

export interface SearchKeysRequest {
  query: string;
}

export interface KeyCollection {
  name: string;
}
