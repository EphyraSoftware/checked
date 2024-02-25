<script setup lang="ts">
import MyKeys from "../trusted/trusted/MyKeys.vue";
import KeyCollections from "../trusted/trusted/KeyCollections.vue";
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
