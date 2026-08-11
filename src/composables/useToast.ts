import { ref } from 'vue'

export type ToastPosition = 'top-right' | 'bottom-right' | 'top-left' | 'bottom-left' | 'top-center' | 'bottom-center'

export interface ToastAction {
  label: string
  onClick: () => void
}

export interface Toast {
  id: number
  message: string
  type: 'success' | 'error' | 'warning' | 'info'
  duration?: number
  position?: ToastPosition
  action?: ToastAction
}

const toasts = ref<Toast[]>([])
let nextId = 1
const MAX_TOASTS = 5

export function useToast() {
  function show(
    message: string,
    type: Toast['type'] = 'info',
    duration = 4000,
    options?: { position?: ToastPosition; action?: ToastAction }
  ) {
    const id = nextId++

    // Remove oldest toast if at limit
    if (toasts.value.length >= MAX_TOASTS) {
      toasts.value.shift()
    }

    const toast: Toast = { id, message, type, duration, ...options }
    toasts.value.push(toast)
    if (duration > 0) {
      setTimeout(() => remove(id), duration)
    }
    return id
  }

  function remove(id: number) {
    const idx = toasts.value.findIndex(t => t.id === id)
    if (idx !== -1) toasts.value.splice(idx, 1)
  }

  function success(message: string, duration?: number, options?: { position?: ToastPosition; action?: ToastAction }) {
    return show(message, 'success', duration, options)
  }

  function error(message: string, duration?: number, options?: { position?: ToastPosition; action?: ToastAction }) {
    return show(message, 'error', duration, options)
  }

  function warning(message: string, duration?: number, options?: { position?: ToastPosition; action?: ToastAction }) {
    return show(message, 'warning', duration, options)
  }

  function info(message: string, duration?: number, options?: { position?: ToastPosition; action?: ToastAction }) {
    return show(message, 'info', duration, options)
  }

  function getToastsByPosition(position: ToastPosition) {
    return toasts.value.filter(t => t.position === position || (!t.position && position === 'bottom-right'))
  }

  return { toasts, show, remove, success, error, warning, info, getToastsByPosition }
}
