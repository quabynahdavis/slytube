<script setup lang="ts">
defineProps<{
  message?: string
  retryable?: boolean
}>()

const emit = defineEmits<{
  retry: []
}>()
</script>

<template>
  <div class="flex flex-col items-center justify-center py-12 px-4 text-center">
    <div class="size-12 rounded-full bg-destructive/10 flex items-center justify-center mb-4">
      <svg class="size-6 text-destructive" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
        <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" />
      </svg>
    </div>
    <h3 class="text-lg font-semibold text-foreground mb-2">Something went wrong</h3>
    <p class="text-sm text-muted-foreground max-w-md mb-4">
      {{ message || 'An unexpected error occurred. Please try again.' }}
    </p>
    <button
      v-if="retryable"
      class="px-4 py-2 bg-primary text-primary-foreground rounded-lg text-sm font-medium hover:bg-primary/90 transition-colors"
      @click="emit('retry')"
    >
      Try Again
    </button>
  </div>
</template>
