<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { parseHostEvent } from '@/types/protocol'
import type { HostEvent } from '@/types/protocol'

const LOAD_TIMEOUT_MS = 15_000

const props = withDefaults(
  defineProps<{
    // Game id; resolves to the static iframe bundle at <gamesBase>/games/<game>/.
    game: string
    // Human-readable game title used for the iframe accessibility label.
    title?: string
    width?: string
    height?: string
    // CSS aspect-ratio used when no explicit height is given.
    aspect?: string
  }>(),
  { title: 'Game', width: '100%', height: '', aspect: '1 / 1' },
)

const emit = defineEmits<{ event: [HostEvent] }>()
const frame = ref<HTMLIFrameElement | null>(null)
const isLoading = ref(true)
const hasError = ref(false)
let loadTimer: ReturnType<typeof setTimeout> | null = null

// When VITE_GAMES_BASE_URL is set the games are served from a separate origin.
// Falls back to same-origin so local dev and the Docker image work unchanged.
const gamesBase = (import.meta.env.VITE_GAMES_BASE_URL ?? '').replace(/\/$/, '')
let gamesOrigin = window.location.origin
if (gamesBase) {
  try {
    gamesOrigin = new URL(gamesBase).origin
  } catch {
    console.warn('GameFrame: invalid VITE_GAMES_BASE_URL, falling back to same-origin')
  }
}

function gameSrc(game: string) {
  const path = `${gamesBase}/games/${game}/index.html`
  // Pass parent origin so host.js can postMessage back to the right target.
  return `${path}?parent=${encodeURIComponent(window.location.origin)}`
}

function startLoadTimer() {
  loadTimer = setTimeout(() => {
    if (isLoading.value) {
      isLoading.value = false
      hasError.value = true
    }
  }, LOAD_TIMEOUT_MS)
}

function cancelLoadTimer() {
  if (loadTimer !== null) {
    clearTimeout(loadTimer)
    loadTimer = null
  }
}

function retry() {
  hasError.value = false
  isLoading.value = true
  cancelLoadTimer()
  if (frame.value) {
    frame.value.src = gameSrc(props.game)
  }
  startLoadTimer()
}

function onMessage(e: MessageEvent) {
  if (e.origin !== gamesOrigin) return
  if (!frame.value || e.source !== frame.value.contentWindow) return
  const data = e.data as Record<string, unknown> | null
  if (!data || data.source !== 'mq-game') return
  const raw = data.event as Record<string, unknown> | null
  if (!raw) return

  const hostEvent = parseHostEvent(raw)
  if (!hostEvent) return

  if (hostEvent.type === 'ready') {
    cancelLoadTimer()
    isLoading.value = false
    hasError.value = false
  }

  emit('event', hostEvent)
}

onMounted(() => {
  window.addEventListener('message', onMessage)
  startLoadTimer()
})

// Reset loading state and restart the timeout when the game prop changes so
// that navigating between games (same PlayView instance, different gameId) gets
// a fresh spinner and error-detection window for each new iframe load.
watch(
  () => props.game,
  () => {
    cancelLoadTimer()
    isLoading.value = true
    hasError.value = false
    startLoadTimer()
  },
)
// The iframe is destroyed by Vue on unmount, which fully tears down the game's
// wasm instance, WebGL context, and animation loop.
onBeforeUnmount(() => {
  window.removeEventListener('message', onMessage)
  cancelLoadTimer()
})

// Send a command into the running game (e.g. pause/reset).
// TODO(server): used by the future multiplayer UI; unverified end-to-end until
// the Rust game server exists.
function post(message: unknown) {
  frame.value?.contentWindow?.postMessage(
    { source: 'mq-host', message },
    gamesOrigin,
  )
}
defineExpose({ post })
</script>

<template>
  <div
    :style="{
      position: 'relative',
      width: props.width,
      height: props.height || undefined,
      aspectRatio: props.height ? undefined : props.aspect,
    }"
  >
    <iframe
      ref="frame"
      :src="gameSrc(props.game)"
      class="game-frame"
      :title="props.title"
    />

    <div v-if="isLoading && !hasError" class="overlay overlay-loading">
      <v-progress-circular indeterminate color="primary" />
    </div>

    <div v-if="hasError" class="overlay overlay-error">
      <p class="text-medium-emphasis">Failed to load game.</p>
      <v-btn size="small" variant="tonal" @click="retry">Retry</v-btn>
    </div>
  </div>
</template>

<style scoped>
.game-frame {
  width: 100%;
  height: 100%;
  border: none;
  display: block;
  border-radius: 4px;
}

.overlay {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
}

.overlay-loading {
  background: #000;
}

.overlay-error {
  flex-direction: column;
  gap: 12px;
  background: #111;
}
</style>
