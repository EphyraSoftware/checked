<script setup lang="ts">
import { defineProps } from 'vue'

defineProps<{
    loading: boolean
}>()

</script>

<template>
    <Transition name="fade" mode="out-in">
        <div class="flex flex-col items-center" v-if="loading">
            <slot name="loading"></slot>
            <span class="loading loading-infinity loading-lg"></span>
        </div>
        <template v-else>
            <slot name="content"></slot>
        </template>
    </Transition>
</template>

<style>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 1s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
