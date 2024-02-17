<script setup lang="ts">
import { AppAgentClient, EntryContent, Record } from '@holochain/client';
import { decode } from '@msgpack/msgpack';
import { ComputedRef, inject, ref, watch } from 'vue';
import { GpgKeyDist } from './types';
import { Snackbar } from '@material/mwc-snackbar';

const client = inject('client') as ComputedRef<AppAgentClient>
const records = ref<GpgKeyDist[]>([])
const fetchError = ref<HTMLElement | null>(null)

const fetch = async () => {
    try {
        console.log('making fetch');
        const r: Record[] = await client.value.callZome({
            role_name: 'trusted',
            zome_name: 'trusted',
            fn_name: 'get_my_keys',
            payload: null,
            cap_secret: null,
        });

        records.value = r.map((record) => {
            return decode((record.entry as any).Present.entry) as GpgKeyDist
        })
    } catch (e: any) {
        const errorSnackbar = fetchError.value as Snackbar;
        errorSnackbar.labelText = `Error loading my keys: ${e.data}`;
        errorSnackbar.show();
    }
}

watch(client, async (client) => {
    if (client) {
        console.log('got client')
        fetch()
    }
}, { immediate: true })

</script>

<template>
    <mwc-snackbar ref="fetchError"></mwc-snackbar>
    <div>
        <h1>My Keys</h1>
        <div v-for="record in records" :key="record.fingerprint">
            <div>{{ record.fingerprint }}</div>
            <div>{{ record.user_id }}</div>
        </div>
    </div>
</template>
