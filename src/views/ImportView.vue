<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { openAndParseCsv, confirmImport } from '@/services/csv_import'
import { getAccounts, getCategories, updateAccountName } from '@/services/getters'
import type { StagedTransaction, Category, Account } from '@/types'

const router = useRouter()

// Staged data (in-memory only — nothing in DB yet)
const stagedRows = ref<StagedTransaction[]>([])
const accountId = ref('')
const categories = ref<Category[]>([])
const skippedRows = ref<string[]>([])
const error = ref('')
const saving = ref(false)

// Account naming — contextual to the imported CSV
const existingAccount = ref<Account | null>(null)
const isNewAccount = computed(() => existingAccount.value === null && accountId.value !== '')
const accountDisplayName = ref('')

// Summary stats
const totalCount = computed(() => stagedRows.value.length)
const categorizedCount = computed(() =>
  stagedRows.value.filter(r => r.category_id !== null).length
)
const uncategorizedCount = computed(() => totalCount.value - categorizedCount.value)

// Format cents to dollar string (e.g. -1599 -> "-$15.99")
function formatCents(cents: number): string {
  const dollars = Math.abs(cents) / 100
  const formatted = dollars.toFixed(2)
  return cents < 0 ? `-$${formatted}` : `$${formatted}`
}

// When user changes a category dropdown on a row
function updateCategory(index: number, value: string) {
  const id = value === '' ? null : Number(value)
  stagedRows.value[index].category_id = id
  stagedRows.value[index].rule_matched = false // user override
}


// Check if the imported csv_id matches an existing account
async function lookupAccount() {
  const accounts = await getAccounts()
  const match = accounts.find(a => a.csv_id === accountId.value)
  existingAccount.value = match || null
  accountDisplayName.value = match?.name || ''
}

// Re-fetch categories from DB (in case user added new ones)
async function refreshCategories() {
  categories.value = await getCategories()
}

async function handleConfirm() {
  saving.value = true
  error.value = ''
  try {
    await confirmImport(accountId.value, stagedRows.value)

    // Save display name if user entered one
    const name = accountDisplayName.value.trim()
    if (name) {
      // Re-fetch accounts to get the row ID (import may have just created it)
      const accounts = await getAccounts()
      const match = accounts.find(a => a.csv_id === accountId.value)
      if (match) await updateAccountName(match.id, name)
    }

    router.replace('/transactions')
  } catch (e) {
    error.value = String(e)
    saving.value = false
  }
}

// Discard everything and go back
function handleCancel() {
  router.replace('/transactions')
}

// Trigger the import dialog as soon as this view loads
onMounted(async () => {
  try {
    const result = await openAndParseCsv()
    if (!result) {
      // User cancelled the file dialog — go back
      router.replace('/transactions')
      return
    }
    accountId.value = result.account_id
    stagedRows.value = result.transactions
    skippedRows.value = result.skipped_rows
    categories.value = await getCategories()
    await lookupAccount()
  } catch (e) {
    error.value = String(e)
  }
})
</script>

<template>
  <div class="view">
    <div class="header-row">
      <h2>Import Preview</h2>
      <div v-if="stagedRows.length > 0" class="import-actions">
        <button class="btn-cancel" @click="handleCancel">Cancel</button>
        <button class="btn-confirm" @click="handleConfirm" :disabled="saving">
          {{ saving ? 'Saving...' : 'Confirm Import' }}
        </button>
      </div>
    </div>

    <!-- Error message -->
    <div v-if="error" class="card error-card">
      {{ error }}
    </div>

    <!-- Loading state (before CSV is parsed) -->
    <div v-else-if="stagedRows.length === 0" class="card">
      <div class="placeholder">Loading CSV...</div>
    </div>

    <!-- Preview table -->
    <template v-else>
      <!-- Summary -->
      <div class="card summary">
        <span><strong>{{ accountId }}</strong></span>
        <span>{{ totalCount }} transactions</span>
        <span class="cat-matched">{{ categorizedCount }} categorized</span>
        <span class="cat-unmatched">{{ uncategorizedCount }} uncategorized</span>
      </div>

      <!-- Account name -->
      <div class="card rename-section">
        <h3>{{ isNewAccount ? 'New Account' : 'Existing Account' }}</h3>

        <div class="rename-row">
          <span class="account-csv-id">{{ accountId }}</span>
          <input
            class="rename-input"
            v-model="accountDisplayName"
            :placeholder="isNewAccount ? 'Give it a name (e.g. Main Checking)' : 'Display name'"
          />
        </div>

        <div v-if="isNewAccount" class="account-hint">
          First time importing from this account.
        </div>
      </div>

      <!-- Skipped rows warning -->
      <div v-if="skippedRows.length > 0" class="card warning-card">
        <strong>{{ skippedRows.length }} row(s) skipped due to parse errors:</strong>
        <ul>
          <li v-for="(msg, i) in skippedRows" :key="i">{{ msg }}</li>
        </ul>
      </div>

      <!-- Table -->
      <div class="table-wrapper">
        <table class="import-table">
          <thead>
            <tr>
              <th>Date</th>
              <th>Description</th>
              <th>Amount</th>
              <th>Type</th>
              <th>Category <button class="btn-refresh" @click="refreshCategories">Refresh</button></th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="(row, index) in stagedRows"
              :key="index"
              :class="{ unmatched: row.category_id === null }"
            >
              <td>{{ row.posting_date }}</td>
              <td class="desc-cell">{{ row.description }}</td>
              <td :class="row.amount_cents >= 0 ? 'amount-pos' : 'amount-neg'">
                {{ formatCents(row.amount_cents) }}
              </td>
              <td>{{ row.transaction_type }}</td>
              <td>
                <select
                  :value="row.category_id ?? ''"
                  @change="updateCategory(index, ($event.target as HTMLSelectElement).value)"
                >
                  <option value="">Uncategorized</option>
                  <option
                    v-for="cat in categories"
                    :key="cat.id"
                    :value="cat.id"
                  >
                    {{ cat.name }}
                  </option>
                </select>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

    </template>
  </div>
</template>

<style scoped>
.summary {
  display: flex;
  gap: 1.5rem;
  align-items: center;
  flex-wrap: wrap;
}

.cat-matched {
  color: var(--accent);
}

.cat-unmatched {
  color: #f59e0b;
}

.error-card {
  color: var(--danger);
  border-color: var(--danger);
}

.warning-card {
  color: #f59e0b;
  border-color: #f59e0b;
}

.warning-card ul {
  margin: 0.4rem 0 0 1.2rem;
  padding: 0;
}

.warning-card li {
  font-size: 0.85rem;
}

.table-wrapper {
  overflow-x: auto;
  margin-bottom: 1rem;
}

.import-table {
  width: 100%;
  border-collapse: collapse;
  background-color: var(--bg-elevated);
  border: 1px solid var(--border-color);
  border-radius: 8px;
}

.import-table th,
.import-table td {
  padding: 0.6rem 0.8rem;
  text-align: left;
  border-bottom: 1px solid var(--border-subtle);
  font-size: 0.9rem;
}

.import-table th {
  color: var(--text-muted);
  font-weight: 500;
  font-size: 0.8rem;
  text-transform: uppercase;
}

.desc-cell {
  max-width: 250px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.amount-pos {
  color: var(--accent);
}

.amount-neg {
  color: var(--text-primary);
}

.unmatched {
  opacity: 0.7;
}


.header-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1rem;
}

.header-row h2 {
  margin-bottom: 0;
}

.import-actions {
  display: flex;
  gap: 0.75rem;
}

.btn-confirm {
  background-color: var(--accent);
  color: #000;
  font-weight: 600;
}

.btn-confirm:hover {
  background-color: var(--accent-hover);
}

.btn-confirm:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-cancel {
  background-color: transparent;
  border: 1px solid var(--border-color);
}

.btn-cancel:hover {
  border-color: var(--danger);
}

.btn-refresh {
  font-size: 0.7rem;
  padding: 0.2em 0.5em;
  margin-left: 0.5rem;
  vertical-align: middle;
}

.rename-section h3 {
  margin-bottom: 0.75rem;
  color: var(--text-secondary);
  font-size: 0.95rem;
}

.rename-row {
  display: flex;
  gap: 0.75rem;
  align-items: center;
  flex-wrap: wrap;
}

.rename-input {
  padding: 0.5rem 0.75rem;
  background-color: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  color: var(--text-primary);
  font-size: 0.85rem;
}

.rename-input:focus {
  border-color: var(--accent);
  outline: none;
}

.account-csv-id {
  font-weight: 600;
  color: var(--text-secondary);
  font-size: 0.9rem;
}

.account-hint {
  margin-top: 0.5rem;
  font-size: 0.8rem;
  color: var(--text-muted);
}

.placeholder {
  color: var(--text-faint);
  padding: 3rem;
  text-align: center;
  border: 1px dashed var(--border-color);
  border-radius: 8px;
}
</style>
