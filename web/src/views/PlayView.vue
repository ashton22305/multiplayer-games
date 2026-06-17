<script setup lang="ts">
import { computed, defineAsyncComponent } from 'vue'
import { games } from '@/data/games'
import GameStats from '@/components/GameStats.vue'

const props = defineProps<{ gameId: string }>()

const game = computed(() => games.find((g) => g.gameId === props.gameId))

const gameComponent = computed(() =>
  game.value ? defineAsyncComponent(game.value.component) : null,
)
</script>

<template>
  <div v-if="game">
    <h1 class="text-h4 mb-4">{{ game.title }}</h1>
    <v-row>
      <v-col cols="12" md="8">
        <component :is="gameComponent" />
      </v-col>
      <v-col cols="12" md="4">
        <GameStats :game-id="game.gameId" />
      </v-col>
    </v-row>
  </div>

  <v-empty-state
    v-else
    icon="mdi-help-circle-outline"
    title="Game not found"
    :text="`There is no game with id &quot;${gameId}&quot;.`"
  >
    <template #actions>
      <v-btn :to="{ name: 'home' }" color="primary">Back to games</v-btn>
    </template>
  </v-empty-state>
</template>
