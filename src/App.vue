<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { Settings } from 'lucide-vue-next'
import { getSetting } from '@/services/settings'
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window'

const route = useRoute()
const showNav = computed(() => route.path !== '/')

// Apply saved theme and window size on app start
onMounted(async () => {
  try {
    const theme = await getSetting('theme')
    if (theme === 'light') {
      document.documentElement.classList.add('light-theme')
    }

    // Apply saved window size
    const width = await getSetting('window_width')
    const height = await getSetting('window_height')
    if (width && height) {
      const win = getCurrentWindow()
      await win.setSize(new LogicalSize(Number(width), Number(height)))
    }
  } catch {
    // DB might not be ready yet on first load — that's fine
  }
})
</script>

<template>
  <div class="app-shell">
    <nav v-if="showNav" class="nav-bar">
      <span class="nav-title">Chase Income Tracker</span>
      <div class="nav-links">
        <RouterLink to="/dashboard">Dashboard</RouterLink>
        <RouterLink to="/categories">Categories</RouterLink>
        <RouterLink to="/transactions">Transactions</RouterLink>
      </div>
      <RouterLink to="/settings" class="nav-settings" aria-label="Settings">
        <Settings :size="20" />
      </RouterLink>
    </nav>

    <main class="app-content">
      <RouterView />
    </main>
  </div>
</template>

<style scoped>
.app-shell {
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
}

.app-content {
  flex: 1;
  overflow: auto;
  height: 0;
}

.nav-bar {
  display: flex;
  align-items: center;
  gap: 1.5rem;
  padding: 0.75rem 1.5rem;
  background-color: var(--bg-surface);
  border-bottom: 1px solid var(--border-color);
}

.nav-title {
  color: var(--accent);
  font-weight: 600;
  font-size: 1.1rem;
}

.nav-links {
  display: flex;
  gap: 1.5rem;
  margin-left: auto;
}

.nav-links a {
  color: var(--text-muted);
  text-decoration: none;
  font-size: 0.9rem;
  transition: color 0.2s;
}

.nav-links a:hover {
  color: var(--text-primary);
}

.nav-links a.router-link-active {
  color: var(--accent);
}

.nav-settings {
  color: var(--text-muted);
  display: flex;
  align-items: center;
  transition: color 0.2s;
}

.nav-settings:hover {
  color: var(--text-primary);
}

.nav-settings.router-link-active {
  color: var(--accent);
}
</style>
