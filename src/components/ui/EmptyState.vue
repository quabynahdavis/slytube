<script setup lang="ts">
import { useSlots } from 'vue'

defineProps<{
  icon?: string
  title?: string
  action?: string
}>()

const emit = defineEmits<{
  action: []
}>()

const slots = useSlots()
</script>

<template>
  <div class="flex flex-col items-center justify-center py-16 px-4 text-center">
    <div v-if="icon || slots.icon" class="size-16 rounded-full bg-muted flex items-center justify-center mb-4">
      <slot name="icon">
        <svg v-if="icon" class="size-8 text-muted-foreground" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
          <path stroke-linecap="round" stroke-linejoin="round" d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2-2v-5m16 0h-2.586a1 1 0 00-.707.293l-2.414 2.414a1 1 0 01-.707.293h-3.172a1 1 0 01-.707-.293l-2.414-2.414A1 1 0 006.586 13H4" />
        </svg>
      </slot>
    </div>
    <h3 v-if="title" class="text-lg font-semibold text-foreground mb-2">{{ title }}</h3>
    <p class="text-sm text-muted-foreground max-w-md mb-4">
      <slot />
    </p>
    <button
      v-if="action"
      class="px-4 py-2 bg-primary text-primary-foreground rounded-lg text-sm font-medium hover:bg-primary/90 transition-colors"
      @click="emit('action')"
    >
      {{ action }}
    </button>
  </div>
</template>
