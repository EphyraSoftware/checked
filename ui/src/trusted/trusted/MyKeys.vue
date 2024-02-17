<script setup lang="ts">
import { useMyKeysStore } from "../../store/my-keys-store";
import KeyList from "../../component/KeyList.vue";

const myKeysStore = useMyKeysStore();
</script>

<template>
  <mwc-snackbar ref="fetchError"></mwc-snackbar>
  <div>
    <h3 class="text-lg">My Keys</h3>
    <KeyList :keys="myKeysStore.myKeys" :readonly="true"></KeyList>
  </div>
</template>
