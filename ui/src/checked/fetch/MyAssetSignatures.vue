<script setup lang="ts">
import LoadingSpinner from "../../component/LoadingSpinner.vue";
import { storeToRefs } from "pinia";
import { useMyAssetSignaturesStore } from "../../store/my-asset-signatures-store";

const { loading, myAssetSignatures } = storeToRefs(useMyAssetSignaturesStore());
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
        <div v-if="myAssetSignatures.length === 0" class="ms-5 italic">
          <p>You haven't created any asset signatures yet.</p>
        </div>
        <template v-else>
          <div
            v-for="c in myAssetSignatures"
            v-bind:key="c.fetch_url"
            class="mt-3"
          >
            <p class="font-bold">{{ c.fetch_url }}</p>
          </div>
        </template>
      </div>
    </template>
  </LoadingSpinner>
</template>
