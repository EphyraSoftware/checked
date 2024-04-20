<script setup lang="ts">
import { useMyKeysStore } from "../../store/my-keys-store";
import KeyList from "../../component/KeyList.vue";
import DistributeSigningKey from "./DistributeSigningKey.vue";
import {computed} from "vue";
import LoadingSpinner from "../../component/LoadingSpinner.vue";
import { storeToRefs } from "pinia";

const { loading, myKeys } = storeToRefs(useMyKeysStore());

// Encourage the user to distribute a key if they don't have any yet
const showDistribute = computed(() => myKeys.value.length === 0);

</script>

<template>
  <div>
    <div class="flex">
      <h3 class="text-lg">My Signing Verification Keys</h3>

      <div class="flex flex-grow justify-end">
        <button
          class="btn btn-circle btn-sm btn-primary"
          :title="
            showDistribute
              ? 'Cancel'
              : 'Distribute your signing verification key'
          "
          @click="showDistribute = !showDistribute"
        >
          <font-awesome-icon
            v-if="!showDistribute"
            icon="fa-regular fa-share-from-square"
          />
          <font-awesome-icon v-else icon="fa-solid fa-xmark" />
        </button>
      </div>
    </div>

    <LoadingSpinner :loading="loading">
      <template #loading>
        <p class="text-lg p-2">Loading keys</p>
      </template>
      <template #content>
        <div>
          <!-- Single root for loading transition -->
          <Transition name="fade">
            <div class="flex justify-center my-3" v-if="showDistribute">
              <div class="card w-3/4 bg-base-100 shadow-xl">
                <div class="card-body">
                  <h2 class="card-title">
                    Distribute your signing verification key
                  </h2>
                  <DistributeSigningKey
                    @distributed="showDistribute = false"
                  ></DistributeSigningKey>
                </div>
              </div>
            </div>
          </Transition>

          <template v-if="myKeys.length">
            <KeyList :key-dist-list="myKeys" :readonly="true"></KeyList>
          </template>
        </div>
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
