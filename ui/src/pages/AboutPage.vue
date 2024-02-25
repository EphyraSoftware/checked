<script setup lang="ts">
import NavBar from '../component/NavBar.vue';

</script>

<template>
    <NavBar></NavBar>
    <div class="container mx-auto mt-5">
        <p>
            You can learn more about protecting your E-mails with GPG from the
            <a href="https://emailselfdefense.fsf.org/en/" target="_blank" class="link">Free Software Foundation</a>.
        </p>
        <p>
            This hApp is a distributed solution for section #5. Rather than
            needing to send your public keys to a centralized server, you can
            distribute them using Holochain.
        </p>

        <br />
        <p>
            You can get the
            <a href="https://github.com/ThetaSinner/hWOT" target="_blank" class="link">source code</a>
            for this app to verify its behavior.
        </p>
    </div>
</template>