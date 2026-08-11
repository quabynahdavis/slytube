import { ref, watch } from 'vue'

export type Theme = 'light' | 'dark' | 'system'

const theme = ref<Theme>((localStorage.getItem('theme') as Theme) || 'system')

function applyTheme(t: Theme) {
  const root = document.documentElement
  const isDark =
    t === 'dark' || (t === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches)
  if (isDark) {
    root.classList.add('dark')
  } else {
    root.classList.remove('dark')
  }
}

watch(theme, (t) => {
  localStorage.setItem('theme', t)
  applyTheme(t)
})

// Listen for system theme changes
if (typeof window !== 'undefined') {
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
    if (theme.value === 'system') applyTheme('system')
  })
  applyTheme(theme.value)
}

export function useTheme() {
  function setTheme(t: Theme) {
    theme.value = t
  }

  return { theme, setTheme }
}
