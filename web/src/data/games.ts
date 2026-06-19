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
  {
    gameId: 'pacman',
    title: 'Pac-Man',
    description: 'Pac-Man remade by me.',
    available: true,
    component: () => import('@/components/PacmanGame.vue'),
  }
]
