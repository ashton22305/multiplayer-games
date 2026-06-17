<script setup lang="ts">
import { RouterView } from 'vue-router'
</script>

<template>
  <v-app>
    <v-app-bar flat>
      <v-btn :to="{ name: 'home' }" variant="text" class="text-h6 text-none">Arcade</v-btn>
      <v-spacer />
      <v-btn :to="{ name: 'home' }" variant="text">Games</v-btn>
    </v-app-bar>

    <v-main>
      <v-container>
        <RouterView />
      </v-container>
    </v-main>

    <v-footer class="justify-center text-medium-emphasis">
      Multiplayer Arcade - built with Vue and Rust/WASM
    </v-footer>
  </v-app>
</template>
