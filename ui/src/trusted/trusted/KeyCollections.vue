<script setup lang="ts">
import { useKeyCollectionsStore } from "../../store/key-collections-store";

const keyCollectionsStore = useKeyCollectionsStore();
</script>

<template>
  <p v-for="c in keyCollectionsStore.keyCollections" v-bind:key="c.name">
    {{ c.name }}, {{ c.gpg_keys.length }}
  </p>
</template>
