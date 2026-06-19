import { describe, expect, it } from 'vitest'
import { games } from './games'

describe('games catalog', () => {
  it('is not empty', () => {
    expect(games.length).toBeGreaterThan(0)
  })

  it('has unique gameIds', () => {
    const ids = games.map((g) => g.gameId)
    expect(new Set(ids).size).toBe(ids.length)
  })

  it('includes snake as an available game', () => {
    const snake = games.find((g) => g.gameId === 'snake')
    expect(snake?.available).toBe(true)
  })

  it('includes pacman as an available game', () => {
    const pacman = games.find((g) => g.gameId === 'pacman')
    expect(pacman?.available).toBe(true)
  })

  it('gives every game a component loader', () => {
    for (const game of games) {
      expect(typeof game.component).toBe('function')
    }
  })
})
