<script setup lang="ts">
import { ref, onErrorCaptured } from 'vue'

const error = ref<Error | null>(null)

onErrorCaptured((err) => {
  error.value = err
  return false
})

function reset() {
  error.value = null
}
</script>

<template>
  <div v-if="error" class="flex flex-col items-center justify-center p-8 text-center">
    <div class="size-16 rounded-full bg-destructive/10 flex items-center justify-center mb-4">
      <svg class="size-8 text-destructive" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
        <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" />
      </svg>
    </div>
    <h2 class="text-lg font-semibold text-foreground mb-2">Something went wrong</h2>
    <p class="text-sm text-muted-foreground mb-4">{{ error.message }}</p>
    <button @click="reset" class="px-4 py-2 bg-primary text-primary-foreground rounded-lg text-sm">
      Try Again
    </button>
  </div>
  <slot v-else />
</template>
