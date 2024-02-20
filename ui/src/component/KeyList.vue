<script setup lang="ts">
import { useMyKeysStore } from "../store/my-keys-store";
import { formatDistanceToNow } from "date-fns";
import { GpgKeyDist } from "../trusted/trusted/types";
import { ref } from "vue";
import GpgFingerprint from "./GpgFingerprint.vue";

defineProps<{
  keys: GpgKeyDist[];
  readonly: boolean;
}>();

const emit = defineEmits<{
  (e: "add-key", key: GpgKeyDist): void;
}>();

const myKeysStore = useMyKeysStore();

const copied = ref<string | null>(null);

const isMine = (keyDist: GpgKeyDist) => {
  return myKeysStore.myKeys.some((r) => r.fingerprint === keyDist.fingerprint);
};

const copyFingerprint = (keyDist: GpgKeyDist) => {
  navigator.clipboard.writeText(keyDist.fingerprint);
  copied.value = keyDist.fingerprint;

  setTimeout(() => {
    copied.value = null;
  }, 1200);
};
</script>

<template>
  <table class="table">
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

        <td class="cursor-pointer font-mono" @click="copyFingerprint(k)">
          <GpgFingerprint :fingerprint="k.fingerprint" v-memo="[k.fingerprint]" />
          <span :class="{ 'ms-2': true, 'text-success': copied === k.fingerprint }">
            <font-awesome-icon v-if="copied === k.fingerprint" icon="fa-regular fa-check-circle" size="lg" fixed-width />
            <font-awesome-icon v-else icon="fa-regular fa-clipboard" size="lg" fixed-width />
          </span>
        </td>
        <td v-if="!readonly">
          <p v-if="isMine(k)" class="font-bold text-primary">Mine</p>
          <div v-else>
            <button class="btn btn-primary" @click="() => emit('add-key', k)">
              Add
            </button>
          </div>
        </td>
      </tr>
    </tbody>
  </table>
</template>
