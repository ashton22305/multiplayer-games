<script setup lang="ts">
import { computed, watch } from 'vue'
import { games } from '@/data/games'
import GameFrame from '@/components/GameFrame.vue'
import GameStats from '@/components/GameStats.vue'
import { useGameStore } from '@/stores/game'

const props = defineProps<{ gameId: string }>()
const store = useGameStore()

const game = computed(() => games.find((g) => g.gameId === props.gameId))

// immediate: true handles initial mount; subsequent fires handle same-route
// navigation (/play/snake → /play/pacman) where Vue Router reuses this instance.
watch(
  () => props.gameId,
  () => { if (game.value) store.reset(game.value.gameId) },
  { immediate: true },
)
</script>

<template>
  <div v-if="game">
    <h1 class="text-h4 mb-4">{{ game.title }}</h1>
    <v-row>
      <v-col cols="12" md="8">
        <v-card class="pa-4 d-flex flex-column align-center">
          <GameFrame
            :game="game.gameId"
            width="100%"
            :aspect="game.aspect"
            :title="game.title"
            @event="store.applyEvent"
          />
          <p class="text-medium-emphasis mt-2">{{ game.instructions }}</p>
        </v-card>
      </v-col>
      <v-col cols="12" md="4">
        <GameStats />
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
      <v-btn :to="{ name: 'games' }" color="primary">Back to games</v-btn>
    </template>
  </v-empty-state>
</template>
