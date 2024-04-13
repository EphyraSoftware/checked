<script setup lang="ts">
import LoadingSpinner from "../../component/LoadingSpinner.vue";
import { storeToRefs } from "pinia";
import {AssetSignatureResponse, useMyAssetSignaturesStore} from "../../store/my-asset-signatures-store";
import { formatDistanceToNow } from "date-fns";
import {computed, ComputedRef} from "vue";

const { loading, myAssetSignatures } = storeToRefs(useMyAssetSignaturesStore());

const myAssetSignaturesSorted: ComputedRef<AssetSignatureResponse[]> = computed(() => {
  const cp = [...myAssetSignatures.value];
  cp.sort((a, b) => {
    return b.created_at - a.created_at;
  });

  return cp;
});
</script>

<template>
  <p class="text-lg">My Fetched Assets</p>

  <LoadingSpinner :loading="loading">
    <template #loading>
      <p class="text-lg p-2">Loading key collections</p>
    </template>
    <template #content>
      <div>
        <!-- Single root for loading transition -->
        <div v-if="myAssetSignaturesSorted.length === 0" class="ms-5 italic">
          <p>You haven't created any asset signatures yet.</p>
        </div>
        <template v-else>
          <table class="table">
            <thead>
              <tr>
                <th>Fetched from URL</th>
                <th>When</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="c in myAssetSignaturesSorted" v-bind:key="c.fetch_url">
                <td>
                  {{ c.fetch_url }}
                </td>
                <td>
                  {{ formatDistanceToNow(new Date(c.created_at / 1000)) }} ago
                </td>
              </tr>
            </tbody>
          </table>
        </template>
      </div>
    </template>
  </LoadingSpinner>
</template>
