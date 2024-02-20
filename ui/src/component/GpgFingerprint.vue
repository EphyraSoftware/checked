<script setup lang="ts">
import { computed } from 'vue';


const props = defineProps<{
    fingerprint: string
}>();

// There's nothing particularly special about this format but it's the same as the GPG CLI prints and
// it's what people are used to looking at for GPG keys. Hopefully that means its a little easier to
// 40 characters grouped together.
const fingerprintDisplay = computed(() => {
    // It's a 160 bit hash, so 20 bytes, in hex is 40 characters
    if (!props.fingerprint || props.fingerprint.length != 40) return '';

    let repr = '';
    for (let i = 0; i < 40; i += 4) {
        repr += props.fingerprint.slice(i, i + 4) + ' ';

        if (i === 20) repr += ' ';
    }

    return repr.trim();
});

</script>

<template>
<span>{{ fingerprintDisplay }}</span>
</template>