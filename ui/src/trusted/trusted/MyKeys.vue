<script setup lang="ts">
import { useMyKeysStore } from "../../store/my-keys-store";
import KeyList from "../../component/KeyList.vue";

const myKeysStore = useMyKeysStore();
</script>

<template>
  <div>
    <h3 class="text-lg">My GPG Keys</h3>
    <KeyList :keys-with-meta="myKeysStore.myKeys" :readonly="true"></KeyList>
  </div>
</template>
