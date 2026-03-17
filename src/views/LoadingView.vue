
<template>
  <div class="loading-screen">
    <div class="header">
      <h1>Chase Income Tracker</h1>
    </div>

    <div class="image-area">
      <img :src="loadingImage" alt="Loading" class="loading-image" />
    </div>

    <!-- Error bar -->
    <p v-if="error" class="error">{{ error }}</p>

    <div class="bottom">
      <div class="progress-track">
        <div class="progress-fill" :style="{ width: progress + '%' }"></div>
      </div>
      <p class="tip">{{ currentTip }}</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { initDatabase } from '@/services/database'
import { getRandomTip, getRandomImage, loadCustomAssets } from '@/services/loading_assets'

const router = useRouter()
const error = ref<string | null>(null)
const progress = ref(0)
const currentTip = ref(getRandomTip())
const loadingImage = ref(getRandomImage())

let tipInterval: ReturnType<typeof setInterval> | null = null
let progressInterval: ReturnType<typeof setInterval> | null = null

onMounted(async () => {
  // Rotate tips
  const tipLoadDuration = 1000 + Math.random() * 3000
  tipInterval = setInterval(() => {
    currentTip.value = getRandomTip()
  }, tipLoadDuration)

  //Loading Bar
  const loadDuration = 1000 + Math.random() * 3000
  const stepMs = 50
  const maxBeforeDone = 90
  const increment = (maxBeforeDone / loadDuration) * stepMs

  progressInterval = setInterval(() => {
    if (progress.value < maxBeforeDone) {
      progress.value = Math.min(progress.value + increment, maxBeforeDone)
    }
  }, stepMs)

  // awating DB or my artificial loading time
  const delay = new Promise(resolve => setTimeout(resolve, loadDuration))

  try {
    await Promise.all([initDatabase(), delay])
    // Load custom tips/images so they're available for future rotations
    await loadCustomAssets()
    // Snap to 100% then navigate
    progress.value = 100
    await new Promise(resolve => setTimeout(resolve, 300))
    router.replace('/dashboard')
  } catch (e) {
    error.value = String(e)
  }
})

onUnmounted(() => {
  if (tipInterval) clearInterval(tipInterval)
  if (progressInterval) clearInterval(progressInterval)
})
</script>

<style scoped>
.loading-screen {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background-color: #0a0a0a;
}

.header {
  padding: 1rem 2rem;
  border-bottom: 1px solid #333;
}

.header h1 {
  font-size: 1.1rem;
  font-weight: 600;
  color: #c8c8c8;
  letter-spacing: 0.05em;
  text-transform: uppercase;
}

.image-area {
  flex: 1;
  min-height: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

.loading-image {
  height: 100%;
  object-fit: cover;
  opacity: 1;
  filter: none;
}

.error {
  color: #ef4444;
  text-align: center;
  padding: 1rem;
}

.bottom {
  padding: 0 2rem 2rem;
}

.progress-track {
  width: 100%;
  height: 18px;
  background-color: #1e1e1e;
  border-radius: 3px;
  overflow: hidden;
  margin-bottom: 0.75rem;
}

.progress-fill {
  height: 100%;
  background-color: #22c55e;
  border-radius: 3px;
  transition: width 0.15s linear;
}

.tip {
  font-size: 1.5rem;
  color: #888;
  font-style: italic;
  text-align: center;
}
</style>
