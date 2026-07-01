// Mirrors crates/protocol/src/lib.rs (the postMessage payload from games).
// Keep in sync with the Rust definitions; serialization round-trips are verified
// by `cargo test -p protocol` (see crates/protocol/src/lib.rs).

export type GameStatus = 'loading' | 'playing' | 'paused' | 'gameOver'

export type HostEvent =
  | { type: 'ready' }
  | { type: 'scoreChanged'; score: number }
  | { type: 'gameOver'; score: number }
  | { type: 'statusChanged'; status: GameStatus }
  | { type: 'playersOnline'; count: number }

// Validate and narrow an unknown postMessage payload to a HostEvent.
// Returns null for unrecognized types or missing/wrong-type fields.
export function parseHostEvent(raw: Record<string, unknown>): HostEvent | null {
  switch (raw.type) {
    case 'ready':
      return { type: 'ready' }
    case 'scoreChanged':
      return typeof raw.score === 'number' ? { type: 'scoreChanged', score: raw.score } : null
    case 'gameOver':
      return typeof raw.score === 'number' ? { type: 'gameOver', score: raw.score } : null
    case 'statusChanged': {
      const VALID: ReadonlySet<string> = new Set(['loading', 'playing', 'paused', 'gameOver'])
      return typeof raw.status === 'string' && VALID.has(raw.status)
        ? { type: 'statusChanged', status: raw.status as GameStatus }
        : null
    }
    case 'playersOnline':
      return typeof raw.count === 'number' ? { type: 'playersOnline', count: raw.count } : null
    default:
      return null
  }
}
