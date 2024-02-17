<script setup lang="ts">
import { useMyKeysStore } from '../../store/my-keys-store';

const myKeysStore = useMyKeysStore()

</script>

<template>
    <mwc-snackbar ref="fetchError"></mwc-snackbar>
    <div>
        <h3>My Keys</h3>
        <div v-for="record in myKeysStore.myKeys" :key="record.fingerprint">
            <div>{{ record.fingerprint }}</div>
            <div>{{ record.user_id }}</div>
        </div>
    </div>
</template>
