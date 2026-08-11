import { ref } from 'vue'

export interface Toast {
  id: number
  message: string
  type: 'success' | 'error' | 'warning' | 'info'
  duration?: number
}

const toasts = ref<Toast[]>([])
let nextId = 1

export function useToast() {
  function show(message: string, type: Toast['type'] = 'info', duration = 4000) {
    const id = nextId++
    toasts.value.push({ id, message, type, duration })
    if (duration > 0) {
      setTimeout(() => remove(id), duration)
    }
    return id
  }

  function remove(id: number) {
    const idx = toasts.value.findIndex(t => t.id === id)
    if (idx !== -1) toasts.value.splice(idx, 1)
  }

  function success(message: string) { return show(message, 'success') }
  function error(message: string) { return show(message, 'error') }
  function warning(message: string) { return show(message, 'warning') }
  function info(message: string) { return show(message, 'info') }

  return { toasts, show, remove, success, error, warning, info }
}
