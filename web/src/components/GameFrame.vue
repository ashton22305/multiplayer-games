<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { parseHostEvent } from '@/types/protocol'
import type { HostEvent } from '@/types/protocol'

const LOAD_TIMEOUT_MS = 15_000

const props = withDefaults(
  defineProps<{
    game: string
    title?: string
    width?: string
    height?: string
    aspect?: string
  }>(),
  { title: 'Game', width: '100%', height: '', aspect: '1 / 1' },
)

const emit = defineEmits<{ event: [HostEvent] }>()
const frame = ref<HTMLIFrameElement | null>(null)
const isLoading = ref(true)
const hasError = ref(false)
let loadTimer: ReturnType<typeof setTimeout> | null = null

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
  const base = `${gamesBase}/games/_shared/index.html`
  return `${base}?game=${game}&parent=${encodeURIComponent(window.location.origin)}`
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

watch(
  () => props.game,
  () => {
    cancelLoadTimer()
    isLoading.value = true
    hasError.value = false
    startLoadTimer()
  },
)

onBeforeUnmount(() => {
  window.removeEventListener('message', onMessage)
  cancelLoadTimer()
})

function post(message: unknown) {
  frame.value?.contentWindow?.postMessage({ source: 'mq-host', message }, gamesOrigin)
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
      :title="props.title"
      style="width: 100%; height: 100%; border: none; display: block;"
    />

    <div
      v-if="isLoading && !hasError"
      style="position: absolute; inset: 0; display: flex; align-items: center; justify-content: center; background: #000;"
    >
      <v-progress-circular indeterminate color="primary" />
    </div>

    <div
      v-if="hasError"
      style="position: absolute; inset: 0; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 12px; background: #111;"
    >
      <p class="text-medium-emphasis">Failed to load game.</p>
      <v-btn size="small" variant="tonal" @click="retry">Retry</v-btn>
    </div>
  </div>
</template>
