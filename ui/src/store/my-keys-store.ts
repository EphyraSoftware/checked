import { defineStore } from "pinia";
import { ComputedRef, inject, ref, watch } from "vue";
import { GpgKeyDist } from "../trusted/trusted/types";
import { AppAgentClient, Record } from "@holochain/client";
import { decode } from '@msgpack/msgpack';

export const useMyKeysStore = defineStore('my-keys', () => {
    const myKeys = ref<GpgKeyDist[]>([]);

    const insertRecord = (record: Record) => {
        myKeys.value.push(decode((record.entry as any).Present.entry) as GpgKeyDist);
    }

    const insertKey = (key: GpgKeyDist) => {
        myKeys.value.push(key);
    }

    const client = inject('client') as ComputedRef<AppAgentClient>;
    const loadKeys = async (client: AppAgentClient) => {
        const r: Record[] = await client.callZome({
            role_name: 'trusted',
            zome_name: 'trusted',
            fn_name: 'get_my_keys',
            payload: null,
            cap_secret: null,
        });

        myKeys.value = [...r.map((record) => {
            return decode((record.entry as any).Present.entry) as GpgKeyDist
        }), ...myKeys.value]
    }

    watch(client, (client) => {
        loadKeys(client)
    }, { immediate: true })

    return {
        myKeys,
        insertRecord,
        insertKey,
    }
})
