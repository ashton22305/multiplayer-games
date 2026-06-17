import type { Component } from 'vue'

export interface GameSummary {
  gameId: string
  title: string
  description: string
  available: boolean
  component: () => Promise<{ default: Component }>
}

export const games: GameSummary[] = [
  {
    gameId: 'snake',
    title: 'Snake',
    description: 'Classic multiplayer Snake. Last snake standing wins.',
    available: true,
    component: () => import('@/components/SnakeGame.vue'),
  },
]
