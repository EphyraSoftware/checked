<script setup lang="ts">
import { defineProps } from 'vue'

defineProps<{
    loading: boolean
}>()

</script>

<template>
    <div class="flex flex-col items-center" v-if="loading">
        <slot name="loading"></slot>
        <span class="loading loading-infinity loading-lg"></span>
    </div>
    <template v-else>
        <slot name="content"></slot>
    </template>
</template>
