<template>
  <v-empty-state
    icon="mdi-map-marker-question-outline"
    title="404"
    text="That page does not exist."
  >
    <template #actions>
      <v-btn :to="{ name: 'home' }" color="primary">Back to games</v-btn>
    </template>
  </v-empty-state>
</template>
