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
    const tag = target.tagName.toLowerCase()
    if (tag === 'input' || tag === 'textarea' || target.isContentEditable) return

    const key = e.key.toLowerCase()

    // Navigation
    if (key === '/') {
      e.preventDefault()
      shortcuts['/']?.()
    } else if (key === 'g') {
      // g then h = home, g then t = trending, etc.
      // For now just register simple shortcuts
    }

    // Player controls (only on watch page)
    if (key === ' ') {
      e.preventDefault()
      shortcuts['space']?.()
    } else if (key === 'arrowleft') {
      e.preventDefault()
      shortcuts['left']?.()
    } else if (key === 'arrowright') {
      e.preventDefault()
      shortcuts['right']?.()
    } else if (key === 'arrowup') {
      e.preventDefault()
      shortcuts['up']?.()
    } else if (key === 'arrowdown') {
      e.preventDefault()
      shortcuts['down']?.()
    } else if (key === 'f') {
      e.preventDefault()
      shortcuts['f']?.()
    } else if (key === 'm') {
      e.preventDefault()
      shortcuts['m']?.()
    } else if (key === 't') {
      e.preventDefault()
      shortcuts['t']?.()
    } else if (key === 'j') {
      e.preventDefault()
      shortcuts['j']?.()
    } else if (key === 'k') {
      e.preventDefault()
      shortcuts['k']?.()
    } else if (key === 'escape') {
      shortcuts['escape']?.()
    } else if ((e.ctrlKey || e.metaKey) && key === 'd') {
      e.preventDefault()
      shortcuts['mod+d']?.()
    } else if ((e.ctrlKey || e.metaKey) && key === 'l') {
      e.preventDefault()
      shortcuts['mod+l']?.()
    } else if ((e.ctrlKey || e.metaKey) && key === 'k') {
      e.preventDefault()
      shortcuts['mod+k']?.()
    }
  }

  onMounted(() => window.addEventListener('keydown', handleKeyDown))
  onUnmounted(() => window.removeEventListener('keydown', handleKeyDown))

  return { register, unregister }
}
