<script setup lang="ts">
import { useToast } from '@/composables/useToast'
import { Toast, ToastProvider, ToastViewport } from '@/components/ui/toast'
import { cn } from '@/lib/utils'

const { toasts, remove } = useToast()

function variantForType(type: string): 'default' | 'destructive' {
  if (type === 'error' || type === 'warning') return 'destructive'
  return 'default'
}

function iconForType(type: string): string {
  switch (type) {
    case 'success':
      return 'M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z'
    case 'error':
      return 'M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z'
    case 'warning':
      return 'M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z'
    default:
      return 'M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z'
  }
}
</script>

<template>
  <ToastProvider>
    <ToastViewport
      :class="
        cn(
          'fixed bottom-0 right-0 z-[100] flex max-h-screen w-full flex-col-reverse p-4 sm:bottom-0 sm:right-0 sm:top-auto sm:flex-col md:max-w-[420px]'
        )
      "
    >
      <TransitionGroup name="toast" tag="div" class="flex flex-col gap-2">
        <Toast
          v-for="toast in toasts"
          :key="toast.id"
          :variant="variantForType(toast.type)"
          :duration="toast.duration"
          @close="remove(toast.id)"
        >
          <div class="flex items-start gap-3">
            <svg
              class="size-5 shrink-0"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path :d="iconForType(toast.type)" />
            </svg>
            <div class="flex-1 text-sm">{{ toast.message }}</div>
          </div>
        </Toast>
      </TransitionGroup>
    </ToastViewport>
  </ToastProvider>
</template>

<style scoped>
.toast-enter-active,
.toast-leave-active {
  transition: all 0.2s ease;
}

.toast-enter-from {
  opacity: 0;
  transform: translateX(100%);
}

.toast-leave-to {
  opacity: 0;
  transform: translateX(100%);
}
</style>
