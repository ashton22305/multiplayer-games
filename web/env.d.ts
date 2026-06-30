/// <reference types="vite/client" />

interface ImportMetaEnv {
  // Base URL for the separately-deployed games Pages project (e.g. https://games.example.com).
  // When unset, iframes are loaded same-origin from /games/<game>/.
  readonly VITE_GAMES_BASE_URL?: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}

declare module '*.vue' {
  import type { DefineComponent } from 'vue'
  const component: DefineComponent<Record<string, unknown>, Record<string, unknown>, unknown>
  export default component
}
