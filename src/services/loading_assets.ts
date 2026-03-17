import { invoke } from '@tauri-apps/api/core'
import { convertFileSrc } from '@tauri-apps/api/core'
import defaultTips from '@/data/loading_screen/tips/default_tips.json'

// Grab all default image files from the loading_screen/images folder at build time
const imageModules = import.meta.glob('@/data/loading_screen/images/*', {
  eager: true,
  import: 'default',
}) as Record<string, string>

const defaultImages = Object.values(imageModules)

// Custom assets loaded at runtime from the user_custom directory
let customTips: string[] = []
let customImageUrls: string[] = []
let customAssetsLoaded = false

// Load custom tips and images from the filesystem
// Called once during app startup (LoadingView)
export async function loadCustomAssets(): Promise<void> {
  if (customAssetsLoaded) return
  try {
    customTips = await invoke<string[]>('get_custom_tips')
  } catch {
    customTips = []
  }
  try {
    // Get the DB path to derive the user_custom directory
    const dbPath = await invoke<string>('get_db_path_string')
    const configDir = dbPath.substring(0, dbPath.lastIndexOf('/'))
    const imagesDir = configDir + '/user_custom/images'

    const filenames = await invoke<string[]>('list_loading_images')
    customImageUrls = filenames.map(name =>
      convertFileSrc(imagesDir + '/' + name)
    )
  } catch {
    customImageUrls = []
  }
  customAssetsLoaded = true
}

function pickRandom<T>(items: T[]): T {
  return items[Math.floor(Math.random() * items.length)]
}

// Returns a single random tip string (defaults + custom merged)
export function getRandomTip(): string {
  const allTips = [...defaultTips, ...customTips]
  return pickRandom(allTips)
}

// Returns a single random image path (defaults + custom merged)
export function getRandomImage(): string {
  const allImages = [...defaultImages, ...customImageUrls]
  return pickRandom(allImages)
}
