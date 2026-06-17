<script setup lang="ts">
defineProps<{ gameId: string }>()
</script>

<template>
  <v-card>
    <v-card-title>Statistics</v-card-title>
    <v-card-text>
      <v-list density="compact" lines="one">
        <v-list-item title="Games played">
          <template #append>—</template>
        </v-list-item>
        <v-list-item title="High score">
          <template #append>—</template>
        </v-list-item>
        <v-list-item title="Players online">
          <template #append>—</template>
        </v-list-item>
      </v-list>

      <div class="text-subtitle-1 mt-4 mb-1">Leaderboard</div>
      <p class="text-medium-emphasis">No results recorded yet.</p>
    </v-card-text>
  </v-card>
</template>
