<script setup lang="ts">
import { AppAgentClient, Record } from '@holochain/client';
import { ComputedRef, inject, ref } from 'vue';
import { GpgKeyDist, SearchKeysRequest } from './types';
import { decode } from '@msgpack/msgpack';
import { useNotificationsStore } from '../../store/notifications';

const searchQuery = ref('');
const searching = ref(false)
const results = ref<GpgKeyDist[]>([]);

const client = inject('client') as ComputedRef<AppAgentClient>;

const notifications = useNotificationsStore();

const searchKeys = async () => {
    if (!client.value || searching.value) return;

    searching.value = true;

    try {
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

        results.value = r.map((record) => decode((record.entry as any).Present.entry) as GpgKeyDist)
    } catch (e) {
        notifications.pushNotification({
            message: `Error searching for keys - ${e}`,
            type: 'error',
            timeout: 5000,
        })
    } finally {
        setTimeout(() => {
            searching.value = false;
        }, 500)
    }
}

</script>

<template>
    <span>Search for keys by user id, email or fingerprint</span>

    <form @submit="e => e.preventDefault()">
        <div class="join">
            <input type="text" class="input input-bordered join-item" placeholder="Search" name="search-for-keys" id="search-for-keys" v-model="searchQuery" />
            <button class="btn join-item min-w-24" @click="searchKeys" :disabled="!searchQuery">
                <span v-if="searching" class="loading loading-spinner"></span>
                <span v-else>Search</span>
            </button>
        </div>
    </form>

    <div v-for="r in results" v-bind:key="r.fingerprint">
        <div class="font-bold">{{ r.fingerprint }}</div>
        <div>{{ r.user_id }}</div>
    </div>
</template>
