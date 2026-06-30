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

  it('gives every game non-empty instructions', () => {
    for (const game of games) {
      expect(game.instructions.length).toBeGreaterThan(0)
    }
  })

  it('gives every game a non-empty title and description', () => {
    for (const game of games) {
      expect(game.title.length).toBeGreaterThan(0)
      expect(game.description.length).toBeGreaterThan(0)
    }
  })

  it('aspect, when set, matches a valid CSS aspect-ratio pattern', () => {
    const ratio = /^\d+ \/ \d+$/
    for (const game of games) {
      if (game.aspect !== undefined) {
        expect(game.aspect).toMatch(ratio)
      }
    }
  })
})
