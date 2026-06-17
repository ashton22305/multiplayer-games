<script setup lang="ts">
import type { GameSummary } from '@/data/games'

defineProps<{ game: GameSummary }>()
</script>

<template>
  <v-card :disabled="!game.available" height="100%" class="d-flex flex-column">
    <v-card-title>{{ game.title }}</v-card-title>
    <v-card-text class="flex-grow-1">{{ game.description }}</v-card-text>
    <v-card-actions>
      <v-btn
        v-if="game.available"
        :to="{ name: 'play', params: { gameId: game.gameId } }"
        color="primary"
        variant="flat"
      >
        Play
      </v-btn>
      <v-chip v-else size="small">Coming soon</v-chip>
    </v-card-actions>
  </v-card>
</template>
