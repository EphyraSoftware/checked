<script setup lang="ts">
import { useMyKeysStore } from "../../store/my-keys-store";
import KeyList from "../../component/KeyList.vue";
import DistributeGpgKey from "./DistributeGpgKey.vue";
import { ref } from "vue";

const myKeysStore = useMyKeysStore();

const showDistribute = ref(false);

</script>

<template>
  <div>
    <div class="flex">
      <h3 class="text-lg">My GPG Keys</h3>
      <div class="flex flex-grow justify-end">
        <button class="btn btn-circle btn-primary" title="Distribute a GPG key" @click="showDistribute = !showDistribute">
          <font-awesome-icon v-if="!showDistribute"
              icon="fa-regular fa-share-from-square"
            />
            <font-awesome-icon v-else
              icon="fa-solid fa-xmark"
            />
        </button>
      </div>
    </div>
    
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

    <KeyList :keys-with-meta="myKeysStore.myKeys" :readonly="true"></KeyList>
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
