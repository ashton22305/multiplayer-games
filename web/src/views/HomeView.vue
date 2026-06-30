<template>
  <div>
    <h1 class="text-h3 mb-4">Multiplayer Arcade</h1>
    <p class="text-body-1 text-medium-emphasis mb-6">
      A collection of games built in Rust and compiled to WebAssembly,
      running live in your browser.
    </p>
    <v-btn :to="{ name: 'games' }" color="primary" size="large">Browse games</v-btn>
  </div>
</template>
