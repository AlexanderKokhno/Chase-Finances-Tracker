<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open, save } from '@tauri-apps/plugin-dialog'
import { openPath, openUrl } from '@tauri-apps/plugin-opener'
import { getAccounts } from '@/services/getters'
import { getSetting, setSetting } from '@/services/settings'
import type { Account } from '@/types'

// ---------- Data Management ----------

const accounts = ref<Account[]>([])
const selectedAccountId = ref<number | null>(null)
const showRebuildConfirm = ref(false)
const showNukeRulesConfirm = ref(false)
const showDeleteAccountConfirm = ref(false)
const showImportConfirm = ref(false)
const pendingImportPath = ref('')
const dbPath = ref('')

// ---------- Loading Screen ----------

const customTips = ref<string[]>([])
const newTipText = ref('')
const customImages = ref<string[]>([])

// ---------- Appearance ----------

const theme = ref('dark')

const windowSizePresets = [
  { label: '2K (2560 x 1440)', value: '2560x1440' },
  { label: '1080p (1920 x 1080)', value: '1920x1080' },
  { label: '1600 x 900', value: '1600x900' },
  { label: '1366 x 768', value: '1366x768' },
  { label: '720p (1280 x 720)', value: '1280x720' },
]
const windowSize = ref('1920x1080')

// ---------- Status messages ----------

const statusMessage = ref('')
const statusType = ref<'success' | 'error'>('success')

function showStatus(message: string, type: 'success' | 'error' = 'success') {
  statusMessage.value = message
  statusType.value = type
  setTimeout(() => { statusMessage.value = '' }, 3000)
}

// ---------- Data Management actions ----------

async function rebuildDatabase() {
  try {
    await invoke('rebuild_database')
  } catch (e) {
    showStatus('Failed to rebuild database: ' + e, 'error')
  }
}

async function openDbFile() {
  try {
    const path = await invoke<string>('get_db_path_string')
    // Open the containing folder, not the file itself
    const folder = path.substring(0, path.lastIndexOf('/'))
    await openPath(folder)
  } catch (e) {
    showStatus('Failed to open database location: ' + e, 'error')
  }
}

async function nukeAllRules() {
  try {
    await invoke('nuke_all_rules')
    showNukeRulesConfirm.value = false
    showStatus('All rules and categories deleted')
  } catch (e) {
    showStatus('Failed to delete rules and categories: ' + e, 'error')
  }
}

async function deleteAccount() {
  if (selectedAccountId.value === null) return
  try {
    await invoke('delete_account', { accountId: selectedAccountId.value })
    showDeleteAccountConfirm.value = false
    selectedAccountId.value = null
    // Refresh accounts list
    accounts.value = await getAccounts()
    showStatus('Account and its transactions deleted')
  } catch (e) {
    showStatus('Failed to delete account: ' + e, 'error')
  }
}

// ---------- Export / Import ----------

async function exportDatabase() {
  try {
    const destPath = await save({
      defaultPath: 'chase_tracker_backup.db',
      filters: [{ name: 'SQLite Database', extensions: ['db'] }]
    })
    if (!destPath) return
    await invoke('export_database', { destPath })
    showStatus('Database exported successfully')
  } catch (e) {
    showStatus('Failed to export database: ' + e, 'error')
  }
}

async function pickImportFile() {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'SQLite Database', extensions: ['db'] }]
    })
    if (!selected) return
    pendingImportPath.value = selected
    showImportConfirm.value = true
  } catch (e) {
    showStatus('Failed to select file: ' + e, 'error')
  }
}

async function confirmImport() {
  try {
    await invoke('import_database', { sourcePath: pendingImportPath.value })
    // App will restart after import, so this line may not run
  } catch (e) {
    showImportConfirm.value = false
    showStatus('Failed to import database: ' + e, 'error')
  }
}

// ---------- Rules actions ----------

const loadingDefaults = ref(false)

async function loadDefaults() {
  loadingDefaults.value = true
  try {
    const result = await invoke<string>('load_default_rules')
    showStatus(result)
  } catch (e) {
    showStatus('Failed to load defaults: ' + e, 'error')
  } finally {
    loadingDefaults.value = false
  }
}

// ---------- Loading Screen actions ----------

async function loadCustomTips() {
  try {
    customTips.value = await invoke<string[]>('get_custom_tips')
  } catch (e) {
    console.error('Failed to load custom tips:', e)
  }
}

async function addTip() {
  const text = newTipText.value.trim()
  if (!text) return
  try {
    await invoke('add_custom_tip', { text })
    newTipText.value = ''
    await loadCustomTips()
  } catch (e) {
    showStatus('Failed to add tip: ' + e, 'error')
  }
}

async function removeTip(index: number) {
  try {
    await invoke('remove_custom_tip', { index })
    await loadCustomTips()
  } catch (e) {
    showStatus('Failed to remove tip: ' + e, 'error')
  }
}

async function loadCustomImages() {
  try {
    customImages.value = await invoke<string[]>('list_loading_images')
  } catch (e) {
    console.error('Failed to load custom images:', e)
  }
}

async function addImage() {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'gif', 'webp'] }]
    })
    if (!selected) return
    await invoke('add_loading_image', { sourcePath: selected })
    await loadCustomImages()
    showStatus('Image added')
  } catch (e) {
    showStatus('Failed to add image: ' + e, 'error')
  }
}

async function removeImage(filename: string) {
  try {
    await invoke('remove_loading_image', { filename })
    await loadCustomImages()
    showStatus('Image removed')
  } catch (e) {
    showStatus('Failed to remove image: ' + e, 'error')
  }
}

// ---------- Complain ----------

const showComplainForm = ref(false)
const complainEmail = ref('')
const complainTitle = ref('')
const complainContent = ref('')

function submitComplaint() {
  const to = 'alexanderkokhnobuilds@gmail.com'
  const subject = encodeURIComponent(complainTitle.value)
  const body = encodeURIComponent(
    `From: ${complainEmail.value}\n\n${complainContent.value}`
  )
  openUrl(`mailto:${to}?subject=${subject}&body=${body}`)
  showComplainForm.value = false
  complainEmail.value = ''
  complainTitle.value = ''
  complainContent.value = ''
  showStatus('Opening your email client...')
}

// ---------- Appearance actions ----------

async function toggleTheme() {
  theme.value = theme.value === 'dark' ? 'light' : 'dark'
  await setSetting('theme', theme.value)
  applyTheme(theme.value)
}

function applyTheme(t: string) {
  document.documentElement.classList.toggle('light-theme', t === 'light')
}

async function saveWindowSize() {
  const [w, h] = windowSize.value.split('x')
  await setSetting('window_width', w)
  await setSetting('window_height', h)
  showStatus('Window size saved — will apply on next launch')
}

// ---------- Init ----------

onMounted(async () => {
  // Load accounts for the delete dropdown
  accounts.value = await getAccounts()

  // Load DB path
  try {
    dbPath.value = await invoke<string>('get_db_path_string')
  } catch (e) {
    console.error('Failed to get DB path:', e)
  }

  // Load custom tips and images
  await loadCustomTips()
  await loadCustomImages()

  // Load saved settings
  const savedTheme = await getSetting('theme')
  if (savedTheme) {
    theme.value = savedTheme
    applyTheme(savedTheme)
  }

  const savedWidth = await getSetting('window_width')
  const savedHeight = await getSetting('window_height')
  if (savedWidth && savedHeight) {
    windowSize.value = savedWidth + 'x' + savedHeight
  }
})
</script>

<template>
  <div class="settings-view">
    <div class="settings-header">
      <h2>Settings</h2>
      <button class="btn-secondary" @click="showComplainForm = true">Complain</button>
    </div>

    <!-- Status toast -->
    <div v-if="statusMessage" :class="['status-toast', statusType]">
      {{ statusMessage }}
    </div>

    <!-- Section 1: Data Management -->
    <section class="settings-section">
      <h3>Data Management</h3>

      <div class="setting-row">
        <div>
          <strong>Database Location</strong>
          <p class="setting-desc">{{ dbPath }}</p>
        </div>
        <button class="btn-secondary" @click="openDbFile">Open Folder</button>
      </div>

      <div class="setting-row">
        <div>
          <strong>Export Database</strong>
          <p class="setting-desc">Save a copy of your database so you can transfer it to another build.</p>
        </div>
        <button class="btn-secondary" @click="exportDatabase">Export</button>
      </div>

      <div class="setting-row">
        <div>
          <strong>Import Database</strong>
          <p class="setting-desc">Replace the current database with a previously exported one. The app will restart.</p>
        </div>
        <button class="btn-secondary" @click="pickImportFile">Import</button>
      </div>

      <div class="setting-row">
        <div>
          <strong>Rebuild Database</strong>
          <p class="setting-desc">Completely deletes the database and restarts the app. All data will be lost.</p>
        </div>
        <button class="btn-danger" @click="showRebuildConfirm = true">Rebuild</button>
      </div>

      <div class="setting-row">
        <div>
          <strong>Nuke All Rules & Categories</strong>
          <p class="setting-desc">Deletes every category rule and category. Use "Load Defaults" to restore them.</p>
        </div>
        <button class="btn-danger" @click="showNukeRulesConfirm = true">Nuke Rules</button>
      </div>

      <div class="setting-row">
        <div>
          <strong>Delete Account</strong>
          <p class="setting-desc">Removes an account and all its transactions permanently.</p>
        </div>
        <div class="account-delete-row">
          <select v-model="selectedAccountId" class="form-select">
            <option :value="null" disabled>Select account...</option>
            <option v-for="acc in accounts" :key="acc.id" :value="acc.id">
              {{ acc.name || acc.csv_id }}
            </option>
          </select>
          <button
            class="btn-danger"
            :disabled="selectedAccountId === null"
            @click="showDeleteAccountConfirm = true"
          >Delete</button>
        </div>
      </div>
    </section>

    <!-- Section 2: Rules -->
    <section class="settings-section">
      <h3>Rules</h3>

      <div class="setting-row">
        <div>
          <strong>Load Default Rules</strong>
          <p class="setting-desc">Imports the built-in default categories and rules. Existing ones are kept.</p>
        </div>
        <button
          class="btn-secondary"
          :disabled="loadingDefaults"
          @click="loadDefaults"
        >
          {{ loadingDefaults ? 'Loading...' : 'Load Defaults' }}
        </button>
      </div>
    </section>

    <!-- Section 3: Loading Screen -->
    <section class="settings-section">
      <h3>Loading Screen</h3>

      <!-- Custom Tips -->
      <div class="setting-block">
        <strong>Custom Tips / Quotes</strong>
        <p class="setting-desc">These show alongside the default tips on the loading screen.</p>

        <div class="tip-add-row">
          <input
            v-model="newTipText"
            class="form-input"
            placeholder="Enter a new tip or quote..."
            @keyup.enter="addTip"
          />
          <button class="btn-secondary" @click="addTip">Add</button>
        </div>

        <div v-if="customTips.length > 0" class="tip-list">
          <div v-for="(tip, i) in customTips" :key="i" class="tip-item">
            <span>{{ tip }}</span>
            <button class="btn-remove" @click="removeTip(i)">Remove</button>
          </div>
        </div>
        <p v-else class="setting-desc">No custom tips added yet.</p>
      </div>

      <!-- Custom Images -->
      <div class="setting-block">
        <strong>Custom Images</strong>
        <p class="setting-desc">Add images that show on the loading screen.</p>

        <button class="btn-secondary" style="margin-top: 0.5rem" @click="addImage">Add Image</button>

        <div v-if="customImages.length > 0" class="tip-list">
          <div v-for="name in customImages" :key="name" class="tip-item">
            <span>{{ name }}</span>
            <button class="btn-remove" @click="removeImage(name)">Remove</button>
          </div>
        </div>
        <p v-else class="setting-desc">No custom images added yet.</p>
      </div>
    </section>

    <!-- Section 4: Appearance -->
    <section class="settings-section">
      <h3>Appearance</h3>

      <div class="setting-row">
        <div>
          <strong>Theme</strong>
          <p class="setting-desc">Switch between dark and light mode.</p>
        </div>
        <button class="btn-secondary" @click="toggleTheme">
          {{ theme === 'dark' ? 'Switch to Light' : 'Switch to Dark' }}
        </button>
      </div>

      <div class="setting-row">
        <div>
          <strong>Default Window Size</strong>
          <p class="setting-desc">Set the window dimensions for the next app launch.</p>
        </div>
        <div class="window-size-row">
          <select v-model="windowSize" @change="saveWindowSize">
            <option v-for="preset in windowSizePresets" :key="preset.value" :value="preset.value">
              {{ preset.label }}
            </option>
          </select>
        </div>
      </div>
    </section>

    <!-- Confirmation Modals -->

    <!-- Rebuild DB -->
    <div v-if="showRebuildConfirm" class="modal-overlay" @click.self="showRebuildConfirm = false">
      <div class="modal-card">
        <h3>Rebuild Database?</h3>
        <p>
          This will <strong>completely delete</strong> your database and restart the app.
          All imported transactions and data will be permanently lost.
        </p>
        <div class="modal-actions">
          <button class="modal-cancel" @click="showRebuildConfirm = false">Cancel</button>
          <button class="modal-confirm" @click="rebuildDatabase">Yes, nuke it</button>
        </div>
      </div>
    </div>

    <!-- Nuke Rules -->
    <div v-if="showNukeRulesConfirm" class="modal-overlay" @click.self="showNukeRulesConfirm = false">
      <div class="modal-card">
        <h3>Delete All Rules & Categories?</h3>
        <p>
          This will delete <strong>every category rule and category</strong>.
          Use "Load Defaults" to restore them, or add new ones manually.
        </p>
        <div class="modal-actions">
          <button class="modal-cancel" @click="showNukeRulesConfirm = false">Cancel</button>
          <button class="modal-confirm" @click="nukeAllRules">Yes, delete all</button>
        </div>
      </div>
    </div>

    <!-- Delete Account -->
    <div v-if="showDeleteAccountConfirm" class="modal-overlay" @click.self="showDeleteAccountConfirm = false">
      <div class="modal-card">
        <h3>Delete Account?</h3>
        <p>
          This will permanently delete the selected account and <strong>all its transactions</strong>.
        </p>
        <div class="modal-actions">
          <button class="modal-cancel" @click="showDeleteAccountConfirm = false">Cancel</button>
          <button class="modal-confirm" @click="deleteAccount">Yes, delete it</button>
        </div>
      </div>
    </div>
    <!-- Import DB -->
    <div v-if="showImportConfirm" class="modal-overlay" @click.self="showImportConfirm = false">
      <div class="modal-card">
        <h3>Import Database?</h3>
        <p>
          This will <strong>replace your current database</strong> with the selected file
          and restart the app. All current data will be overwritten.
        </p>
        <div class="modal-actions">
          <button class="modal-cancel" @click="showImportConfirm = false">Cancel</button>
          <button class="modal-confirm" @click="confirmImport">Yes, import it</button>
        </div>
      </div>
    </div>

    <!-- Complain -->
    <div v-if="showComplainForm" class="modal-overlay" @click.self="showComplainForm = false">
      <div class="modal-card complain-modal">
        <h3 class="complain-title">Send Feedback</h3>
        <p>This will open your email client.</p>

        <label class="complain-label">Your Email</label>
        <input v-model="complainEmail" class="form-input complain-input" placeholder="you@example.com" />

        <label class="complain-label">Subject</label>
        <input v-model="complainTitle" class="form-input complain-input" placeholder="What's wrong?" />

        <label class="complain-label">Details</label>
        <textarea v-model="complainContent" class="form-input complain-textarea" placeholder="Tell me about it..." rows="4"></textarea>

        <div class="modal-actions">
          <button class="modal-cancel" @click="showComplainForm = false">Cancel</button>
          <button class="btn-secondary complain-send" :disabled="!complainTitle.trim()" @click="submitComplaint">Send</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-view {
  padding: 1.5rem;
  max-width: 800px;
  margin: 0 auto;
}

.settings-section {
  background-color: var(--bg-surface);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 1.25rem;
  margin-bottom: 1.5rem;
}

.settings-section h3 {
  margin: 0 0 1rem;
  color: var(--text-secondary);
  text-transform: uppercase;
  font-size: 0.85rem;
  letter-spacing: 0.05em;
}

.setting-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.75rem 0;
  border-bottom: 1px solid var(--border-subtle);
}

.setting-row:last-child {
  border-bottom: none;
}

.setting-block {
  padding: 0.75rem 0;
  border-bottom: 1px solid var(--border-subtle);
}

.setting-block:last-child {
  border-bottom: none;
}

.setting-desc {
  color: var(--text-faint);
  font-size: 0.8rem;
  margin: 0.25rem 0 0;
}

/* Buttons */
.btn-secondary {
  padding: 0.4rem 0.8rem;
  background: var(--bg-input);
  color: var(--text-secondary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  font-size: 0.8rem;
  cursor: pointer;
  white-space: nowrap;
}

.btn-secondary:hover {
  border-color: var(--accent);
  color: var(--text-primary);
}

.btn-secondary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-danger {
  padding: 0.4rem 0.8rem;
  background: transparent;
  color: #ef4444;
  border: 1px solid #ef4444;
  border-radius: 6px;
  font-size: 0.8rem;
  cursor: pointer;
  white-space: nowrap;
}

.btn-danger:hover {
  background: #ef4444;
  color: #fff;
}

.btn-danger:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-remove {
  padding: 0.2rem 0.5rem;
  background: transparent;
  color: var(--danger);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  font-size: 0.75rem;
  cursor: pointer;
}

.btn-remove:hover {
  border-color: var(--danger);
}

/* Forms */
.form-input {
  padding: 0.4rem 0.6rem;
  background: var(--bg-input);
  color: var(--text-primary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  font-size: 0.85rem;
}

.form-input:focus {
  outline: none;
  border-color: var(--accent);
}

.account-delete-row {
  display: flex;
  gap: 0.5rem;
  align-items: center;
}

/* Tips */
.tip-add-row {
  display: flex;
  gap: 0.5rem;
  margin-top: 0.75rem;
}

.tip-add-row .form-input {
  flex: 1;
}

.tip-list {
  margin-top: 0.75rem;
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.tip-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.4rem 0.6rem;
  background: var(--bg-input);
  border-radius: 4px;
  font-size: 0.85rem;
  color: var(--text-secondary);
}

/* Window size */
.window-size-row {
  display: flex;
  align-items: center;
}

/* Status toast */
.status-toast {
  position: fixed;
  top: 4rem;
  right: 1.5rem;
  padding: 0.6rem 1rem;
  border-radius: 6px;
  font-size: 0.85rem;
  z-index: 200;
}

.status-toast.success {
  background: #22c55e;
  color: #111;
}

.status-toast.error {
  background: #ef4444;
  color: #fff;
}

/* Modals */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.modal-card {
  background: var(--bg-elevated);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  padding: 1.5rem 2rem;
  max-width: 420px;
  width: 90%;
}

.modal-card h3 {
  margin: 0 0 0.75rem;
  color: var(--danger);
  text-transform: none;
  font-size: 1.1rem;
  letter-spacing: 0;
}

.modal-card p {
  color: var(--text-secondary);
  font-size: 0.9rem;
  line-height: 1.5;
  margin: 0 0 1.25rem;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.75rem;
}

.modal-cancel {
  padding: 0.45rem 1rem;
  border-radius: 6px;
  border: 1px solid var(--border-color);
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  font-size: 0.85rem;
}

.modal-cancel:hover {
  border-color: var(--text-faint);
}

.modal-confirm {
  padding: 0.45rem 1rem;
  border-radius: 6px;
  border: none;
  background: #dc2626;
  color: white;
  cursor: pointer;
  font-size: 0.85rem;
}

.modal-confirm:hover {
  background: #ef4444;
}

/* ---------- Settings header ---------- */
.settings-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1rem;
}

.settings-header h2 {
  margin-bottom: 0;
}

/* ---------- Complain modal ---------- */
.complain-modal {
  max-width: 480px;
}

.complain-title {
  color: var(--text-primary) !important;
}

.complain-label {
  display: block;
  font-size: 0.8rem;
  color: var(--text-muted);
  text-transform: uppercase;
  margin-bottom: 0.3rem;
  margin-top: 0.75rem;
  font-weight: 500;
}

.complain-input {
  width: 100%;
  box-sizing: border-box;
}

.complain-textarea {
  width: 100%;
  box-sizing: border-box;
  resize: vertical;
  font-family: inherit;
}

.complain-send {
  background: var(--accent) !important;
  color: #111 !important;
  border-color: var(--accent) !important;
  font-weight: 600;
}

.complain-send:hover {
  opacity: 0.9;
}

.complain-send:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
