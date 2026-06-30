import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useGameStore } from './game'

beforeEach(() => {
  setActivePinia(createPinia())
  localStorage.clear()
})

describe('useGameStore', () => {
  describe('reset', () => {
    it('sets status to loading', () => {
      const store = useGameStore()
      store.applyEvent({ type: 'ready' })
      expect(store.status).toBe('playing')

      store.reset('snake')
      expect(store.status).toBe('loading')
    })

    it('clears score', () => {
      const store = useGameStore()
      store.reset('snake')
      store.applyEvent({ type: 'scoreChanged', score: 50 })
      expect(store.score).toBe(50)

      store.reset('snake')
      expect(store.score).toBe(0)
    })

    it('loads persisted high score from localStorage', () => {
      localStorage.setItem('highscore:snake', '200')
      const store = useGameStore()
      store.reset('snake')
      expect(store.highScore).toBe(200)
    })

    it('high score starts at 0 with no localStorage entry', () => {
      const store = useGameStore()
      store.reset('snake')
      expect(store.highScore).toBe(0)
    })
  })

  describe('per-game high score isolation', () => {
    it('does not bleed high score across games', () => {
      const store = useGameStore()

      store.reset('snake')
      store.applyEvent({ type: 'scoreChanged', score: 150 })
      expect(store.highScore).toBe(150)

      // Switch to a different game — snake's high score must not appear.
      store.reset('pacman')
      expect(store.highScore).toBe(0)
    })

    it('preserves each game high score independently', () => {
      const store = useGameStore()

      store.reset('snake')
      store.applyEvent({ type: 'scoreChanged', score: 100 })

      store.reset('pacman')
      store.applyEvent({ type: 'scoreChanged', score: 250 })

      // Switch back: snake's score is still 100.
      store.reset('snake')
      expect(store.highScore).toBe(100)

      store.reset('pacman')
      expect(store.highScore).toBe(250)
    })

    it('persists high score to localStorage per game', () => {
      const store = useGameStore()
      store.reset('snake')
      store.applyEvent({ type: 'scoreChanged', score: 77 })

      expect(localStorage.getItem('highscore:snake')).toBe('77')
      expect(localStorage.getItem('highscore:pacman')).toBeNull()
    })
  })

  describe('applyEvent', () => {
    it("'ready' sets status to playing", () => {
      const store = useGameStore()
      store.reset('snake')
      store.applyEvent({ type: 'ready' })
      expect(store.status).toBe('playing')
    })

    it("'scoreChanged' updates score and high score", () => {
      const store = useGameStore()
      store.reset('snake')
      store.applyEvent({ type: 'scoreChanged', score: 5 })
      expect(store.score).toBe(5)
      expect(store.highScore).toBe(5)
    })

    it("'scoreChanged' does not lower high score", () => {
      const store = useGameStore()
      store.reset('snake')
      store.applyEvent({ type: 'scoreChanged', score: 10 })
      store.applyEvent({ type: 'scoreChanged', score: 3 })
      expect(store.score).toBe(3)
      expect(store.highScore).toBe(10)
    })

    it("'gameOver' sets status, score, and high score", () => {
      const store = useGameStore()
      store.reset('snake')
      store.applyEvent({ type: 'gameOver', score: 42 })
      expect(store.status).toBe('gameOver')
      expect(store.score).toBe(42)
      expect(store.highScore).toBe(42)
    })

    it("'gameOver' score overrides a stale scoreChanged value", () => {
      const store = useGameStore()
      store.reset('snake')
      store.applyEvent({ type: 'scoreChanged', score: 80 })
      store.applyEvent({ type: 'gameOver', score: 100 })
      expect(store.score).toBe(100)
      expect(store.highScore).toBe(100)
    })

    it("'statusChanged' overrides status", () => {
      const store = useGameStore()
      store.reset('snake')
      store.applyEvent({ type: 'ready' })
      store.applyEvent({ type: 'statusChanged', status: 'paused' })
      expect(store.status).toBe('paused')
    })

    it("'playersOnline' sets player count", () => {
      const store = useGameStore()
      store.reset('snake')
      store.applyEvent({ type: 'playersOnline', count: 7 })
      expect(store.playersOnline).toBe(7)
    })
  })
})
