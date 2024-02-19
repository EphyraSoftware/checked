<script setup lang="ts">
import { useKeyCollectionsStore } from "../../store/key-collections-store";
import KeyList from "../../component/KeyList.vue";

const keyCollectionsStore = useKeyCollectionsStore();
</script>

<template>
  <p class="text-lg">My Key Collections</p>
  <div
    v-for="c in keyCollectionsStore.keyCollections"
    v-bind:key="c.name"
    class="mt-3"
  >
    <p class="font-bold">{{ c.name }}</p>
    <KeyList :keys="c.gpg_keys" :readonly="true"></KeyList>
  </div>
</template>
