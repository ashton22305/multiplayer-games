<script setup lang="ts">
import { games } from '@/data/games'
import GameCard from '@/components/GameCard.vue'
</script>

<template>
  <div>
    <h1 class="text-h4 mb-2">Games</h1>
    <p class="text-medium-emphasis mb-6">Pick a game to play. More on the way.</p>
    <v-row>
      <v-col v-for="game in games" :key="game.gameId" cols="12" sm="6" md="4">
        <GameCard :game="game" />
      </v-col>
    </v-row>
  </div>
</template>
