<script setup lang="ts">
defineProps<{
  count?: number
  columns?: number
}>()
</script>

<template>
  <div class="grid gap-4" :style="{ gridTemplateColumns: `repeat(${columns || 3}, 1fr)` }">
    <div v-for="i in (count || 6)" :key="i" class="animate-pulse">
      <div class="aspect-video rounded-lg bg-muted mb-2"></div>
      <div class="flex gap-3">
        <div class="size-9 rounded-full bg-muted shrink-0"></div>
        <div class="flex-1 space-y-2">
          <div class="h-4 bg-muted rounded w-3/4"></div>
          <div class="h-3 bg-muted rounded w-1/2"></div>
          <div class="h-3 bg-muted rounded w-1/4"></div>
        </div>
      </div>
    </div>
  </div>
</template>
