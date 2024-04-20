<script setup lang="ts">
import { useMyKeysStore } from "../store/my-keys-store";
import { VfKeyResponse } from "../checked/signing_keys/types";

defineProps<{
  keyDistList: VfKeyResponse[];
  readonly: boolean;
}>();

const emit = defineEmits<{
  (e: "add-key", key: VfKeyResponse): void;
}>();

const myKeysStore = useMyKeysStore();

const isMine = (key: VfKeyResponse) => {
  return myKeysStore.myKeys.some(
    (r) => r.key_dist_address === key.key_dist_address,
  );
};
</script>

<template>
  <table class="table">
    <thead>
      <tr>
        <th>Name</th>
        <!--<th>Expiry</th>-->
        <th v-if="keyDistList.some((k) => k.reference_count !== undefined)">
          References
        </th>
        <th v-if="!readonly">Action</th>
      </tr>
    </thead>
    <tbody>
      <tr v-for="k in keyDistList" v-bind:key="k.key_dist_address.join('')">
        <td>{{ k.verification_key_dist.name }}</td>
        <!--<td>
          {{
            k.verification_key_dist.expires_at
              ? formatDistanceToNow(k.verification_key_dist.expires_at)
              : "-"
          }}
        </td>-->
        <td>
          {{ k.reference_count }}
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
