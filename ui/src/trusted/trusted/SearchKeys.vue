<script setup lang="ts">
import { AppAgentClient, Record } from '@holochain/client';
import { ComputedRef, inject, ref } from 'vue';
import { GpgKeyDist, SearchKeysRequest } from './types';
import { decode } from '@msgpack/msgpack';
import { useNotificationsStore } from '../../store/notifications-store';
import { useMyKeysStore } from '../../store/my-keys-store';

const searchQuery = ref('');
const searching = ref(false)
const results = ref<GpgKeyDist[]>([]);

const client = inject('client') as ComputedRef<AppAgentClient>;

const notifications = useNotificationsStore();
const myKeysStore = useMyKeysStore();

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

const isMine = (keyDist: GpgKeyDist) => {
    return myKeysStore.myKeys.some((r) => r.fingerprint === keyDist.fingerprint);
}

</script>

<template>
    <p>Search for keys by user id, email or fingerprint.</p>
    <p class="text-sm italic">An exact match is required for all fields and the fingerprint should be in upper-case hex.</p>

    <form @submit="e => e.preventDefault()">
        <div class="join flex w-full">
            <input type="text" class="input input-bordered join-item grow" placeholder="Search" name="search-for-keys"
                id="search-for-keys" v-model="searchQuery" />
            <button class="btn btn-primary join-item min-w-24" @click="searchKeys" :disabled="!searchQuery">
                <span v-if="searching" class="loading loading-spinner"></span>
                <span v-else>Search</span>
            </button>
        </div>
    </form>

    <div class="mt-5">
        <p>Search results</p>

        <table class="table">
            <!-- head -->
            <thead>
                <tr>
                    <th>User ID</th>
                    <th>Email</th>
                    <th>Fingerprint</th>
                    <th>Action</th>
                </tr>
            </thead>
            <tbody>
                <tr v-for="r in results" v-bind:key="r.fingerprint">
                    <td>{{ r.user_id }}</td>
                    <td>{{ r.email ?? '-' }}</td>
                    <td>{{ r.fingerprint }}</td>
                    <td>
                        <p v-if="isMine(r)" class="font-bold text-primary">Mine</p>
                        <div v-else>
                            <button class="btn btn-primary" @click="() => { }">Add</button>
                        </div>
                    </td>
                </tr>
            </tbody>
        </table>
    </div>
</template>
