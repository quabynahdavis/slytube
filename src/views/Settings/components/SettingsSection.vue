<script setup lang="ts">
defineProps<{
  label: string
  description?: string
  icon?: string
}>()
</script>

<template>
  <section class="rounded-xl border border-border bg-card overflow-hidden">
    <div class="px-5 py-4 border-b border-border bg-muted/30">
      <h2 class="text-base font-semibold text-foreground">{{ label }}</h2>
      <p v-if="description" class="text-xs text-muted-foreground mt-0.5">{{ description }}</p>
    </div>
    <div class="divide-y divide-border">
      <slot />
    </div>
  </section>
</template>
