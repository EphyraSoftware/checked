<script setup lang="ts">
import { useMyKeysStore } from "../store/my-keys-store";
import { formatDistanceToNow } from "date-fns";
import { GpgKeyDist, GpgKeyWithMeta } from "../trusted/trusted/types";
import { ref } from "vue";
import GpgFingerprint from "./GpgFingerprint.vue";

defineProps<{
  keysWithMeta: GpgKeyWithMeta[];
  readonly: boolean;
}>();

const emit = defineEmits<{
  (e: "add-key", key: GpgKeyWithMeta): void;
}>();

const myKeysStore = useMyKeysStore();

const copied = ref<string | null>(null);

const isMine = (keyDist: GpgKeyDist) => {
  return myKeysStore.myKeys.some(
    (r) => r.gpg_key_dist.fingerprint === keyDist.fingerprint,
  );
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
        <th v-if="keysWithMeta.some((k) => k.reference_count !== undefined)">
          References
        </th>
        <th>Fingerprint</th>
        <th v-if="!readonly">Action</th>
      </tr>
    </thead>
    <tbody>
      <tr v-for="k in keysWithMeta" v-bind:key="k.gpg_key_dist.fingerprint">
        <td>{{ k.gpg_key_dist.name }}</td>
        <td>{{ k.gpg_key_dist.email ?? "-" }}</td>
        <td>
          {{
            k.gpg_key_dist.expires_at
              ? formatDistanceToNow(k.gpg_key_dist.expires_at)
              : "-"
          }}
        </td>
        <td v-if="keysWithMeta.some((k) => k.reference_count !== undefined)">
          {{ k.reference_count ?? "unknown" }}
        </td>

        <td
          class="cursor-pointer font-mono"
          @click="copyFingerprint(k.gpg_key_dist)"
        >
          <GpgFingerprint :fingerprint="k.gpg_key_dist.fingerprint" />
          <span
            :class="{
              'ms-2': true,
              'text-success': copied === k.gpg_key_dist.fingerprint,
            }"
          >
            <font-awesome-icon
              v-if="copied === k.gpg_key_dist.fingerprint"
              icon="fa-regular fa-check-circle"
              size="lg"
              fixed-width
            />
            <font-awesome-icon
              v-else
              icon="fa-regular fa-clipboard"
              size="lg"
              fixed-width
            />
          </span>
        </td>
        <td v-if="!readonly">
          <p v-if="isMine(k.gpg_key_dist)" class="font-bold text-primary">
            Mine
          </p>
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
