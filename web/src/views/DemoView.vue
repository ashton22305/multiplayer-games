<script setup lang="ts">
import { onMounted } from 'vue'
import GameFrame from '@/components/GameFrame.vue'
import { useGameStore } from '@/stores/game'

const game = useGameStore()
onMounted(() => game.reset('demo'))
</script>

<template>
  <div style="max-width: 640px; margin: 0 auto">
    <h1 class="text-h4 mb-4">Engine Demo</h1>
    <GameFrame game="demo" title="Engine Demo" @event="game.applyEvent" />
    <p class="text-medium-emphasis mt-2">
      Bounces: {{ game.score }} &mdash; resize the window to see the sprite scale.
    </p>
  </div>
</template>
