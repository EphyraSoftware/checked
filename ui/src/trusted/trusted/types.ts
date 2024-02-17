import { 
  SignedActionHashed,
  Create,
  Update,
  Delete,
  CreateLink,
  DeleteLink
} from '@holochain/client';

export type TrustedSignal = {
  type: 'EntryCreated';
  action: SignedActionHashed<Create>;
  app_entry: EntryTypes;
} | {
  type: 'EntryUpdated';
  action: SignedActionHashed<Update>;
  app_entry: EntryTypes;
  original_app_entry: EntryTypes;
} | {
  type: 'EntryDeleted';
  action: SignedActionHashed<Delete>;
  original_app_entry: EntryTypes;
} | {
  type: 'LinkCreated';
  action: SignedActionHashed<CreateLink>;
  link_type: string;
} | {
  type: 'LinkDeleted';
  action: SignedActionHashed<DeleteLink>;
  link_type: string;
};

export type EntryTypes =
 | ({  type: 'GpgKeyDist'; } & GpgKeyDist);

export interface DistributeGpgKeyRequest {
  public_key: string;
}

export interface GpgKeyDist { 
  public_key: string;
  fingerprint: string;
  name: string;
  email?: string;
  expires_at?: Date;
}

export interface SearchKeysRequest {
  query: string;
}
