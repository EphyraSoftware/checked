<script setup lang="ts">
import { useKeyCollectionsStore } from '../../store/key-collections-store';

console.log('creating keyCollectionsStore in KeyCollections.vue')
const keyCollectionsStore = useKeyCollectionsStore();

</script>

<template>
<p v-for="c in keyCollectionsStore.keyCollections">{{ c.name }}</p>
</template>