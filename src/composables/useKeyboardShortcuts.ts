import { onMounted, onUnmounted } from 'vue'

type ShortcutHandler = () => void

export function useKeyboardShortcuts() {
  const shortcuts: Record<string, ShortcutHandler> = {}

  function register(key: string, handler: ShortcutHandler) {
    shortcuts[key] = handler
  }

  function unregister(key: string) {
    delete shortcuts[key]
  }

  function handleKeyDown(e: KeyboardEvent) {
    const target = e.target as HTMLElement
    if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable) return

    const key = e.key.toLowerCase()

    if (key === '/') {
      e.preventDefault()
      shortcuts['/']?.()
    } else if (key === 'j') {
      e.preventDefault()
      shortcuts['j']?.()
    } else if (key === 'k') {
      e.preventDefault()
      shortcuts['k']?.()
    } else if (key === 'l') {
      e.preventDefault()
      shortcuts['l']?.()
    } else if (key === 'f') {
      e.preventDefault()
      shortcuts['f']?.()
    } else if (key === 'm') {
      e.preventDefault()
      shortcuts['m']?.()
    } else if (key === 't') {
      e.preventDefault()
      shortcuts['t']?.()
    } else if ((e.ctrlKey || e.metaKey) && key === 'd') {
      e.preventDefault()
      shortcuts['mod+d']?.()
    } else if ((e.ctrlKey || e.metaKey) && key === 't') {
      e.preventDefault()
      shortcuts['mod+t']?.()
    } else if ((e.ctrlKey || e.metaKey) && key === 'l') {
      e.preventDefault()
      shortcuts['mod+l']?.()
    }
  }

  onMounted(() => window.addEventListener('keydown', handleKeyDown))
  onUnmounted(() => window.removeEventListener('keydown', handleKeyDown))

  return { register, unregister }
}
