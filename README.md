# Multiplayer Arcade

A collection of games written in Rust, compiled to WebAssembly, and embedded in a
Vue 3 portal. Each game runs inside a sandboxed `<iframe>`; the Vue host communicates
with it over `postMessage`.

## Architecture

```
crates/
  protocol/   Shared serde types (HostEvent, GameStatus, …) — Rust + WASM
  engine/     macroquad-based runtime: frame loop, Gfx, Input, CollisionWorld, host bridge
  games/
    snake/    Single-player Snake
    pacman/   Single-player Pac-Man
    demo/     Bouncing-ball engine smoke test (dev only)
web/
  src/
    data/games.ts       Game catalog: id, title, description, aspect, instructions
    stores/game.ts      Pinia store: per-game score/high-score/status (localStorage)
    views/PlayView.vue  Generic play page — renders any catalog game via GameFrame
    components/
      GameFrame.vue     iframe wrapper: origin-safe postMessage, load timeout, error UI
      GameStats.vue     Stats sidebar — reads from the game store
  public/games/_shared/ Vendored macroquad JS loader + host bridge (host.js)
  scripts/build-games.mjs  Cargo → WASM → per-game iframe bundle
```

**Deploy:** static SPA on Cloudflare Pages; WASM bundles on a second Cloudflare Pages
project (or same-origin); future game server on Oracle Cloud Always-Free ARM behind Caddy.
See `docs/deployment.md` for details.

## Quick start

```sh
# 1. Build the WASM game bundles (requires Rust + wasm32-unknown-unknown target)
cd web && npm run build:games

# 2. Start the Vue dev server
npm run dev           # http://localhost:5173
```

The dev server proxies `/api` and `/ws` to `localhost:8080` (for the future game server).
Without the server, all games work offline; the WebSocket path is simply unused.

## Tests

```sh
# Rust (engine + protocol)
cargo test

# TypeScript / Vue
cd web && npm test

# Type check
cd web && npm run typecheck
```

`cargo test -p protocol` includes serialization round-trips that verify the JSON wire
format matches `web/src/types/protocol.ts`. If you change the serde attributes on any
protocol type, update the TS file and the tests together.

## CI / CD

Two deploy pipelines, each with a CI gate:

| Workflow | Trigger | Environment |
|---|---|---|
| `deploy-web-dev.yml` | push to `main` | Cloudflare Pages `main` branch (dev URL) |
| `deploy-web-prod.yml` | push to `prod` | Cloudflare Pages production (custom domain) |
| `deploy-games-dev.yml` | push to `main` (Rust/script changes) | Games Pages project, dev |
| `deploy-games-prod.yml` | push to `prod` | Games Pages project, production |

CI runs `cargo fmt --check`, `cargo clippy`, `cargo test`, then TypeScript
typecheck + Vitest + Vite build. Deploy is gated on `vars.DEPLOY_ENABLED == 'true'`
(flip this once infra is ready).
