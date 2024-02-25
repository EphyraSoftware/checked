<script setup lang="ts">
import { useMyKeysStore } from "../../store/my-keys-store";
import KeyList from "../../component/KeyList.vue";
import DistributeGpgKey from "./DistributeGpgKey.vue";
import { computed, ref, watch } from "vue";
import LoadingSpinner from "../../component/LoadingSpinner.vue";
import { storeToRefs } from "pinia";

const { loading, myKeys } = storeToRefs(useMyKeysStore());

const showDistribute = ref(false);

// Encourage the user to distribute a key if they don't have any yet
watch(loading, (loading) => {
  if (!loading && myKeys.value.length === 0) {
    showDistribute.value = true;
  }
});

</script>

<template>
  <div>
    <div class="flex">
      <h3 class="text-lg">My GPG Keys</h3>

      <div class="flex flex-grow justify-end">
        <button class="btn btn-circle btn-primary" title="Distribute a GPG key" @click="showDistribute = !showDistribute">
          <font-awesome-icon v-if="!showDistribute" icon="fa-regular fa-share-from-square" />
          <font-awesome-icon v-else icon="fa-solid fa-xmark" />
        </button>
      </div>
    </div>

    <LoadingSpinner :loading="loading">
      <template #loading>
        <p class="text-lg p-2">Loading keys</p>
      </template>
      <template #content>
        <Transition name="fade">
          <div class="flex justify-center my-3" v-if="showDistribute">
            <div class="card w-3/4 bg-base-100 shadow-xl">
              <div class="card-body">
                <h2 class="card-title">Distribute your GPG public key</h2>
                <DistributeGpgKey @distributed="showDistribute = false"></DistributeGpgKey>
              </div>
            </div>
          </div>
        </Transition>

        <template v-if="myKeys.length">
          <KeyList :keys-with-meta="myKeys" :readonly="true"></KeyList>
        </template>
      </template>
    </LoadingSpinner>
  </div>
</template>

<style>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.5s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
