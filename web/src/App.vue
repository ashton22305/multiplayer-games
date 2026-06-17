<script setup lang="ts">
import { RouterView } from 'vue-router'
</script>

<template>
  <v-app>
    <v-app-bar flat>
      <v-btn :to="{ name: 'home' }" variant="text" class="text-h6 text-none">Home</v-btn>
      <v-btn :to="{ name: 'games' }" variant="text" class="text-h6 text-none">Games</v-btn>
      <v-spacer />
      <v-btn :href="'https://github.com/ashton22305/multiplayer-games'" target="_blank" variant="text" class="text-h6 text-none">Source</v-btn>
    </v-app-bar>

    <v-main>
      <v-container>
        <RouterView />
      </v-container>
    </v-main>

    <v-footer class="justify-center text-medium-emphasis">
      Multiplayer games built with Vue and Rust/WASM
    </v-footer>
  </v-app>
</template>
