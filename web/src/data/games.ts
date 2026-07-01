export interface GameSummary {
  gameId: string
  title: string
  description: string
  available: boolean
  /** CSS aspect-ratio for the game iframe. Defaults to 1/1 when omitted. */
  aspect?: string
  /** Brief instructions shown below the game canvas on the play page. */
  instructions: string
}

export const games: GameSummary[] = [
  {
    gameId: 'snake',
    title: 'Snake',
    description: 'Classic multiplayer Snake. Last snake standing wins.',
    available: true,
    instructions: 'Arrow keys / WASD to steer. Eat the food; avoid the walls and yourself.',
  },
  {
    gameId: 'pacman',
    title: 'Pac-Man',
    description: 'Pac-Man remade by me.',
    available: true,
    aspect: '19 / 21',
    instructions: 'Arrow keys / WASD to move. Eat the pellets; grab a power pellet to turn the tables on the ghosts.',
  },
]
