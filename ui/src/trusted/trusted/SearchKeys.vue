<script setup lang="ts">
import { AppAgentClient, Record } from '@holochain/client';
import { ComputedRef, inject, ref, watch } from 'vue';
import { GpgKeyDist, SearchKeysRequest } from './types';
import { decode, encode } from '@msgpack/msgpack';

const searchQuery = ref('');
const results = ref<GpgKeyDist[]>([]);

const client = inject('client') as ComputedRef<AppAgentClient>;

const searchKeys = async () => {
    if (!client.value) return;

    const request: SearchKeysRequest = {
        query: searchQuery.value,
    }

    const r: Record[] = await client.value.callZome({
        role_name: 'trusted',
        zome_name: 'trusted',
        fn_name: 'search_keys',
        payload: request,
        cap_secret: null,
    });

    results.value = r.map((record) => {
        console.log('got record', record);
        return decode((record.entry as any).Present.entry) as GpgKeyDist
    })
}

</script>

<template>
    <label for="search-for-keys">Search for keys by user id, email or fingerprint</label>
    <input type="text" name="search-for-keys" id="search-for-keys" v-model="searchQuery" />
    <mwc-button raised label="Search" @click="searchKeys"></mwc-button>

    <div v-for="r in results" v-bind:key="r.fingerprint">
        <div>{{ r.fingerprint }}</div>
        <div>{{ r.user_id }}</div>
    </div>
</template>