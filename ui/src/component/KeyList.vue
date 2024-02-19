<script setup lang="ts">
import { useMyKeysStore } from "../store/my-keys-store";
import { formatDistanceToNow } from "date-fns";
import { GpgKeyDist } from "../trusted/trusted/types";

defineProps<{
  keys: GpgKeyDist[];
  readonly: boolean;
}>();

const emit = defineEmits<{
  (e: 'add-key', key: GpgKeyDist): void;
}>();

const myKeysStore = useMyKeysStore();

const isMine = (keyDist: GpgKeyDist) => {
  return myKeysStore.myKeys.some((r) => r.fingerprint === keyDist.fingerprint);
};
</script>

<template>
  <table class="table">
    <!-- head -->
    <thead>
      <tr>
        <th>Name</th>
        <th>Email</th>
        <th>Expiry</th>
        <th>Fingerprint</th>
        <th v-if="!readonly">Action</th>
      </tr>
    </thead>
    <tbody>
      <tr v-for="k in keys" v-bind:key="k.fingerprint">
        <td>{{ k.name }}</td>
        <td>{{ k.email ?? "-" }}</td>
        <td>{{ k.expires_at ? formatDistanceToNow(k.expires_at) : "-" }}</td>
        <td>{{ k.fingerprint }}</td>
        <td v-if="!readonly">
          <p v-if="isMine(k)" class="font-bold text-primary">Mine</p>
          <div v-else>
            <button class="btn btn-primary" @click="() => emit('add-key', k)">Add</button>
          </div>
        </td>
      </tr>
    </tbody>
  </table>
</template>
