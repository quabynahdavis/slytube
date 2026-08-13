import { ref } from 'vue'

const colorCache = ref<Map<string, string>>(new Map())

function getFallbackColor(): string {
  return 'rgba(142, 142, 142, 0.15)'
}

function extractColorFromImage(img: HTMLImageElement): string {
  const canvas = document.createElement('canvas')
  const ctx = canvas.getContext('2d', { willReadFrequently: true })
  if (!ctx) return getFallbackColor()

  canvas.width = 32
  canvas.height = 32

  ctx.drawImage(img, 0, 0, 32, 32)

  let r = 0, g = 0, b = 0
  const imageData = ctx.getImageData(0, 0, 32, 32)
  const data = imageData.data
  const pixelCount = 32 * 32

  for (let i = 0; i < data.length; i += 4) {
    r += data[i]
    g += data[i + 1]
    b += data[i + 2]
  }

  r = Math.floor(r / pixelCount)
  g = Math.floor(g / pixelCount)
  b = Math.floor(b / pixelCount)

  return `rgba(${r}, ${g}, ${b}, 0.15)`
}

async function processImage(url: string): Promise<string> {
  return new Promise((resolve) => {
    const img = new Image()
    img.crossOrigin = 'anonymous'

    img.onload = () => {
      try {
        const color = extractColorFromImage(img)
        colorCache.value.set(url, color)
        resolve(color)
      } catch {
        resolve(getFallbackColor())
      }
    }

    img.onerror = () => {
      resolve(getFallbackColor())
    }

    img.src = url
  })
}

export function useThumbnailColor() {
  function getColor(url: string | undefined): string {
    if (!url) return getFallbackColor()

    const cached = colorCache.value.get(url)
    if (cached) return cached

    processImage(url).catch(() => getFallbackColor())

    return getFallbackColor()
  }

  return { getColor }
}
