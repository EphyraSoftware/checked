<script setup lang="ts">
import { useKeyCollectionsStore } from "../../store/key-collections-store";
import KeyList from "../../component/KeyList.vue";

const keyCollectionsStore = useKeyCollectionsStore();
</script>

<template>
  <p class="text-lg">My Key Collections</p>
  
  <div v-if="keyCollectionsStore.keyCollections.length === 0" class="ms-5 italic">
    <p>You don't have any key collections yet.</p>
    <!-- TODO I want to link to the search screen here but I need the router for that -->
    <p>When you add a key you will be able to create a collection for it!</p>
  </div>
  <template v-else>
    <div
      v-for="c in keyCollectionsStore.keyCollections"
      v-bind:key="c.name"
      class="mt-3"
    >
      <p class="font-bold">{{ c.name }}</p>
      <KeyList :keys="c.gpg_keys" :readonly="true"></KeyList>
    </div>
  </template>
</template>
