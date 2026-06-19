<script setup lang="ts">
import { ref } from 'vue'

const canvas = ref<HTMLCanvasElement | null>(null)
</script>

<template>
  <v-card class="pa-4 d-flex flex-column align-center">
    <canvas ref="canvas" width="640" height="640" class="pacman-canvas"></canvas>
    <p class="text-medium-emphasis mt-2">The Pac-Man game module will load here.</p>
  </v-card>
</template>
