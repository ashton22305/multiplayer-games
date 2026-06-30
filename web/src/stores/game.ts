import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import type { GameStatus, HostEvent } from '@/types/protocol'

// Live state for the active game, fed by HostEvents from the running wasm game
// (via GameFrame) and read by the stats sidebar.
//
// High scores are keyed by gameId and persisted to localStorage so they survive
// page reloads. Call reset(gameId) whenever a new game is mounted; it loads the
// stored high score for that game and clears transient per-round state.
export const useGameStore = defineStore('game', () => {
  const activeGameId = ref('')
  const status = ref<GameStatus>('loading')
  const score = ref(0)
  const playersOnline = ref(0)

  // Per-game high scores, keyed by gameId.
  const highScores = ref<Record<string, number>>({})

  const highScore = computed(() => highScores.value[activeGameId.value] ?? 0)

  function reset(gameId: string) {
    activeGameId.value = gameId
    status.value = 'loading'
    score.value = 0
    playersOnline.value = 0
    // Hydrate high score from localStorage for this game.
    if (!highScores.value[gameId]) {
      const stored = localStorage.getItem(`highscore:${gameId}`)
      if (stored !== null) {
        highScores.value[gameId] = Number(stored) || 0
      }
    }
  }

  function updateHighScore(n: number) {
    const id = activeGameId.value
    if (!id) return
    if (n > (highScores.value[id] ?? 0)) {
      highScores.value[id] = n
      localStorage.setItem(`highscore:${id}`, String(n))
    }
  }

  function applyEvent(e: HostEvent) {
    switch (e.type) {
      case 'ready':
        status.value = 'playing'
        break
      case 'scoreChanged':
        score.value = e.score
        updateHighScore(e.score)
        break
      case 'gameOver':
        status.value = 'gameOver'
        score.value = e.score
        updateHighScore(e.score)
        break
      case 'statusChanged':
        status.value = e.status
        break
      case 'playersOnline':
        playersOnline.value = e.count
        break
    }
  }

  return { status, score, highScore, playersOnline, reset, applyEvent }
})
