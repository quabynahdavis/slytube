import { ref, type Directive, type Ref, type App } from 'vue'

export interface ScrollAnimationOptions {
  threshold?: number
  rootMargin?: string
  once?: boolean
}

export interface ScrollAnimationReturn {
  isVisible: Ref<boolean>
  scrollAnim: Directive<HTMLElement, undefined>
  staggeredAnim: Directive<HTMLElement, number>
}

// Shared observer state for all instances
let sharedObserver: IntersectionObserver | null = null
const observedElements = new Map<HTMLElement, { hasAnimated: boolean }>()

const defaultOptions: Required<ScrollAnimationOptions> = {
  threshold: 0.1,
  rootMargin: '0px 0px -50px 0px',
  once: true,
}

function createObserver(options: Required<ScrollAnimationOptions>): IntersectionObserver | null {
  if (typeof window === 'undefined' || !('IntersectionObserver' in window)) {
    return null
  }

  if (!sharedObserver) {
    sharedObserver = new IntersectionObserver(
      (entries: IntersectionObserverEntry[]) => {
        for (const entry of entries) {
          const el = entry.target as HTMLElement
          const state = observedElements.get(el)

          if (entry.isIntersecting) {
            if (state) {
              state.hasAnimated = true
            }

            // Apply visible styles
            el.style.opacity = '1'
            el.style.transform = 'translateY(0)'

            // If once mode, stop observing after first intersection
            if (options.once) {
              sharedObserver?.unobserve(el)
            }
          }
        }
      },
      {
        threshold: options.threshold,
        rootMargin: options.rootMargin,
      },
    )
  }

  return sharedObserver
}

/**
 * Composable that provides scroll-triggered fade-in animations using Intersection Observer.
 * Elements fade in and slide up when they enter the viewport.
 *
 * Install globally via the plugin in main.ts:
 * ```ts
 * import { scrollAnimationPlugin } from '@/composables/useScrollAnimation'
 * app.use(scrollAnimationPlugin)
 * ```
 *
 * Then use directives in templates:
 * ```vue
 * <template>
 *   <div v-for="(item, index) in items" :key="item.id" v-staggered-anim="index">
 *     <VideoCard :video="item" />
 *   </div>
 * </template>
 * ```
 */
export function useScrollAnimation(options: ScrollAnimationOptions = {}): ScrollAnimationReturn {
  const mergedOptions: Required<ScrollAnimationOptions> = {
    ...defaultOptions,
    ...options,
  }

  const isVisible = ref(false)

  const ensureObserver = () => {
    const observer = createObserver(mergedOptions)
    if (!observer) {
      isVisible.value = true
      return null
    }
    return observer
  }

  // Base scroll animation directive
  const scrollAnim: Directive<HTMLElement, undefined> = {
    mounted(el) {
      ensureObserver()

      // Start with the element hidden
      el.style.opacity = '0'
      el.style.transform = 'translateY(20px)'
      el.style.transition = 'opacity 300ms ease-out, transform 300ms ease-out'
      el.style.willChange = 'opacity, transform'

      // If no Intersection Observer support, show immediately
      if (!sharedObserver) {
        el.style.opacity = '1'
        el.style.transform = 'translateY(0)'
        return
      }

      observedElements.set(el, { hasAnimated: false })
      sharedObserver.observe(el)
    },
    unmounted(el) {
      sharedObserver?.unobserve(el)
      observedElements.delete(el)
    },
  }

  // Staggered animation directive with cascading delay
  // Usage: v-staggered-anim="index" where index is the item's position
  const staggeredAnim: Directive<HTMLElement, number> = {
    mounted(el, binding) {
      ensureObserver()

      const index = binding.value ?? 0
      const delay = index * 50 // 50ms stagger between items

      // Start with the element hidden
      el.style.opacity = '0'
      el.style.transform = 'translateY(20px)'
      el.style.transition = `opacity 300ms ease-out ${delay}ms, transform 300ms ease-out ${delay}ms`
      el.style.willChange = 'opacity, transform'

      // If no Intersection Observer support, show immediately
      if (!sharedObserver) {
        el.style.opacity = '1'
        el.style.transform = 'translateY(0)'
        return
      }

      observedElements.set(el, { hasAnimated: false })
      sharedObserver.observe(el)
    },
    unmounted(el) {
      sharedObserver?.unobserve(el)
      observedElements.delete(el)
    },
  }

  return {
    isVisible,
    scrollAnim,
    staggeredAnim,
  }
}

/**
 * Plugin to register scroll animation directives globally.
 * Install in main.ts:
 * ```ts
 * import { scrollAnimationPlugin } from './composables/useScrollAnimation'
 * app.use(scrollAnimationPlugin)
 * ```
 */
export const scrollAnimationPlugin = {
  install(app: App) {
    app.directive('scroll-anim', useScrollAnimation().scrollAnim)
    app.directive('staggered-anim', useScrollAnimation().staggeredAnim)
  },
}
