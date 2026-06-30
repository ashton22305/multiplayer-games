<script setup lang="ts">
import { computed } from 'vue'
import { useGameStore } from '@/stores/game'

const game = useGameStore()

const statusLabel = computed(
  () =>
    ({
      loading: 'Loading',
      playing: 'Playing',
      paused: 'Paused',
      gameOver: 'Game over',
    })[game.status] ?? game.status,
)
</script>

<template>
  <v-card>
    <v-card-title>Statistics</v-card-title>
    <v-card-text>
      <v-list density="compact" lines="one">
        <v-list-item title="Status">
          <template #append>{{ statusLabel }}</template>
        </v-list-item>
        <v-list-item title="Score">
          <template #append>{{ game.score }}</template>
        </v-list-item>
        <v-list-item title="High score">
          <template #append>{{ game.highScore }}</template>
        </v-list-item>
        <v-list-item title="Players online">
          <template #append>{{ game.playersOnline || '—' }}</template>
        </v-list-item>
      </v-list>

      <div class="text-subtitle-1 mt-4 mb-1">Leaderboard</div>
      <p class="text-medium-emphasis">No results recorded yet.</p>
    </v-card-text>
  </v-card>
</template>
