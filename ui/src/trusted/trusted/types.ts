import {
  SignedActionHashed,
  Create,
  Update,
  Delete,
  CreateLink,
  DeleteLink,
  ActionHash,
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
  key_type: "MiniSignEd25519";
  proof: string;
  proof_signature: string;
}

export interface VerificationKeyDist {
  verification_key: string;
  key_type: VfKeyType;
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

export type VfKeyType = { MiniSignEd25519: null };

export type MarkVfKeyDistOpt =
  | {
      Rotated: {
        new_verification_key_dist_address: ActionHash;
      };
    }
  | {
      Compromised: {
        note: string;
        since: number;
      };
    };

export interface VfKeyDistResponse {
  verification_key: string;
  key_type: VfKeyType;
  name: string;
  expires_at?: Date;
  marks: MarkVfKeyDistOpt[];
}

export interface VfKeyResponse {
  verification_key_dist: VfKeyDistResponse;
  key_dist_address: ActionHash;
  reference_count: number;
  created_at: number;
}

export interface SearchKeysRequest {
  query: string;
}

export interface KeyCollection {
  name: string;
}
