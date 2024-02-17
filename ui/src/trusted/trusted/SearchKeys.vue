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
    <span>Search for keys by user id, email or fingerprint</span>
    <label class="input input-bordered flex items-center gap-2">
        <input type="text" class="grow" placeholder="Search" name="search-for-keys" id="search-for-keys" v-model="searchQuery" />
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" fill="currentColor" class="w-4 h-4 opacity-70"><path fill-rule="evenodd" d="M9.965 11.026a5 5 0 1 1 1.06-1.06l2.755 2.754a.75.75 0 1 1-1.06 1.06l-2.755-2.754ZM10.5 7a3.5 3.5 0 1 1-7 0 3.5 3.5 0 0 1 7 0Z" clip-rule="evenodd" /></svg>
    </label>
    <button class="btn" @click="searchKeys">Search</button>

    <div v-for="r in results" v-bind:key="r.fingerprint">
        <div class="font-bold">{{ r.fingerprint }}</div>
        <div>{{ r.user_id }}</div>
    </div>
</template>
