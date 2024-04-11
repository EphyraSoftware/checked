<script setup lang="ts">
import MyKeys from "../checked/signing_keys/MyKeys.vue";
import KeyCollections from "../checked/signing_keys/KeyCollections.vue";
import NavBar from "../component/NavBar.vue";
</script>

<template>
  <NavBar></NavBar>
  <div class="container mx-auto mt-5">
    <MyKeys></MyKeys>
    <div class="mt-5">
      <KeyCollections></KeyCollections>
    </div>
  </div>
</template>
