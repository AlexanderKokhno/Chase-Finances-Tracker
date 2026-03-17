<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { ChevronUp, ChevronDown } from 'lucide-vue-next'
import { getAccounts, getCategories, getTransactions, getCategoryRules, updateTransactionCategory } from '@/services/getters'
import { ruleMatchesTransaction } from '@/services/category_matcher'
import { useAccountRename } from '@/composables/useAccountRename'
import type { Transaction } from '@/types'

const router = useRouter()

// ---------- Types ----------

interface FilterOption {
  id: number
  label: string
}

// ---------- State ----------

const rows = ref<Transaction[]>([])
const loading = ref(true)
const error = ref('')
// Filter options loaded from DB
const accountOptions = ref<FilterOption[]>([])
const categoryOptions = ref<FilterOption[]>([])

// Currently selected filter IDs
const selectedAccountIds = ref<number[]>([])
const selectedCategoryIds = ref<number[]>([])

// Search
const searchQuery = ref('')

// Dropdown open/close state
const accountDropdownOpen = ref(false)
const categoryDropdownOpen = ref(false)

// Sort state
type SortColumn = 'posting_date' | 'description' | 'amount_cents' | 'transaction_type' | 'category'
const sortColumn = ref<SortColumn>('posting_date')
const sortDirection = ref<'asc' | 'desc'>('desc')

// Rename account
const { renameAccountId, renameAccountName, renameStatus, onRenameAccountSelect, submitRename } = useAccountRename(accountOptions)

// Apply rules
const applyingRules = ref(false)
const applyRulesStatus = ref('')

// ---------- Helpers ----------

function formatCents(cents: number): string {
  const dollars = Math.abs(cents) / 100
  const formatted = dollars.toFixed(2)
  return cents < 0 ? `-$${formatted}` : `$${formatted}`
}

function toggleSelection(list: number[], id: number) {
  const idx = list.indexOf(id)
  if (idx === -1) {
    list.push(id)
  } else {
    list.splice(idx, 1)
  }
}

function selectionLabel(selected: number[], options: FilterOption[], placeholder: string): string {
  if (selected.length === 0) return placeholder
  if (selected.length === 1) {
    const match = options.find(o => o.id === selected[0])
    return match ? match.label : placeholder
  }
  return `${selected.length} selected`
}

// Sort helpers
function toggleSort(column: SortColumn) {
  if (sortColumn.value === column) {
    // Same column clicked — flip direction
    sortDirection.value = sortDirection.value === 'asc' ? 'desc' : 'asc'
  } else {
    // New column — set it and default to ascending
    sortColumn.value = column
    sortDirection.value = 'asc'
  }
}

const sortedRows = computed(() => {
  const col = sortColumn.value
  const dir = sortDirection.value === 'asc' ? 1 : -1
  const query = searchQuery.value.toLowerCase().trim()

  const filtered = query
    ? rows.value.filter(r => r.description.toLowerCase().includes(query))
    : rows.value

  return [...filtered].sort((a, b) => {
    let valA: string | number
    let valB: string | number

    if (col === 'category') {
      valA = categoryName(a.category_id).toLowerCase()
      valB = categoryName(b.category_id).toLowerCase()
    } else if (col === 'amount_cents') {
      valA = a.amount_cents
      valB = b.amount_cents
    } else if (col === 'description' || col === 'transaction_type') {
      valA = a[col].toLowerCase()
      valB = b[col].toLowerCase()
    } else {
      // posting_date — string comparison works for YYYY-MM-DD or MM/DD/YYYY
      valA = a[col]
      valB = b[col]
    }

    if (valA < valB) return -1 * dir
    if (valA > valB) return 1 * dir
    return 0
  })
})

// Close dropdowns when clicking outside
function onClickOutside(event: MouseEvent) {
  const target = event.target as HTMLElement
  if (!target.closest('.multiselect')) {
    accountDropdownOpen.value = false
    categoryDropdownOpen.value = false
  }
}

// ---------- Apply category rules ----------

async function applyRules() {
  applyingRules.value = true
  applyRulesStatus.value = ''
  try {
    const rules = await getCategoryRules()
    // Load ALL transactions (no filters) so we don't miss any
    const allTxns = await getTransactions()
    let updated = 0

    for (const tx of allTxns) {
      let matchedCategoryId: number | null = null
      for (const rule of rules) {
        if (ruleMatchesTransaction(rule, tx)) {
          matchedCategoryId = rule.category_id
          break
        }
      }

      // Only update if the category actually changed
      if (matchedCategoryId !== tx.category_id) {
        await updateTransactionCategory(tx.id, matchedCategoryId)
        updated++
      }
    }

    applyRulesStatus.value = `Done — ${updated} transaction${updated === 1 ? '' : 's'} updated`
    // Refresh the table to show new categories
    await loadFilterOptions()
    await loadTransactions()
  } catch (e) {
    applyRulesStatus.value = 'Error: ' + String(e)
  } finally {
    applyingRules.value = false
  }
}

// ---------- Data loading ----------

async function loadFilterOptions() {
  try {

    const accounts = await getAccounts()
    accountOptions.value = accounts.map(a => ({
      id: a.id,
      label: a.name || a.csv_id
    }))

    const categories = await getCategories()
    categoryOptions.value = categories.map(c => ({
      id: c.id,
      label: c.name
    }))
  } catch (e) {
    error.value = 'Failed to load filter options: ' + String(e)
  }
}

// Look up category name from the already-loaded filter options
function categoryName(id: number | null): string {
  if (id === null) return 'Uncategorized'
  const match = categoryOptions.value.find(c => c.id === id)
  return match ? match.label : 'Uncategorized'
}

async function loadTransactions() {
  loading.value = true
  error.value = ''
  try {
    rows.value = await getTransactions(
      selectedAccountIds.value,
      selectedCategoryIds.value
    )
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

// Re-fetch transactions whenever filters change
watch([selectedAccountIds, selectedCategoryIds], loadTransactions, { deep: true })

onMounted(async () => {
  document.addEventListener('click', onClickOutside)
  await loadFilterOptions()
  await loadTransactions()
})
</script>

<template>
  <div class="view layout">
    <!-- Left control panel -->
    <aside class="control-panel">
      <!-- Search -->
      <div class="filter-group">
        <label class="filter-label">Search</label>
        <input
          class="search-input"
          v-model="searchQuery"
          placeholder="Search descriptions..."
        />
      </div>

      <h3>Filters</h3>

      <!-- Account filter -->
      <div class="filter-group">
        <label class="filter-label">Account</label>
        <div class="multiselect" @click.stop>
          <button
            class="multiselect-toggle"
            @click="accountDropdownOpen = !accountDropdownOpen"
          >
            {{ selectionLabel(selectedAccountIds, accountOptions, 'All accounts') }}
            <span class="arrow">{{ accountDropdownOpen ? '▲' : '▼' }}</span>
          </button>
          <div v-if="accountDropdownOpen" class="multiselect-menu">
            <label
              v-for="opt in accountOptions"
              :key="opt.id"
              class="multiselect-item"
            >
              <input
                type="checkbox"
                :checked="selectedAccountIds.includes(opt.id)"
                @change="toggleSelection(selectedAccountIds, opt.id)"
              />
              {{ opt.label }}
            </label>
            <div v-if="accountOptions.length === 0" class="multiselect-empty">
              No accounts found
            </div>
          </div>
        </div>
      </div>

      <!-- Category filter -->
      <div class="filter-group">
        <label class="filter-label">Category</label>
        <div class="multiselect" @click.stop>
          <button
            class="multiselect-toggle"
            @click="categoryDropdownOpen = !categoryDropdownOpen"
          >
            {{ selectionLabel(selectedCategoryIds, categoryOptions, 'All categories') }}
            <span class="arrow">{{ categoryDropdownOpen ? '▲' : '▼' }}</span>
          </button>
          <div v-if="categoryDropdownOpen" class="multiselect-menu">
            <label
              v-for="opt in categoryOptions"
              :key="opt.id"
              class="multiselect-item"
            >
              <input
                type="checkbox"
                :checked="selectedCategoryIds.includes(opt.id)"
                @change="toggleSelection(selectedCategoryIds, opt.id)"
              />
              {{ opt.label }}
            </label>
            <div v-if="categoryOptions.length === 0" class="multiselect-empty">
              No categories found
            </div>
          </div>
        </div>
      </div>

      <!-- Rename account -->
      <div class="rename-section">
        <h3>Rename Account</h3>

        <div class="filter-group">
          <label class="filter-label">Account</label>
          <select
            v-model="renameAccountId"
            @change="onRenameAccountSelect"
          >
            <option :value="null" disabled>Select account</option>
            <option v-for="opt in accountOptions" :key="opt.id" :value="opt.id">
              {{ opt.label }}
            </option>
          </select>
        </div>

        <div v-if="renameAccountId" class="filter-group">
          <label class="filter-label">Display Name</label>
          <input
            class="rename-input"
            v-model="renameAccountName"
            @keyup.enter="submitRename"
          />
        </div>

        <button
          v-if="renameAccountId"
          class="rename-btn"
          @click="submitRename"
        >
          Update
        </button>

        <div v-if="renameStatus" class="rename-status">{{ renameStatus }}</div>
      </div>
    </aside>

    <!-- Right content area -->
    <main class="content">
      <div class="header-row">
        <h2>Transactions</h2>
        <div class="header-actions">
          <span v-if="applyRulesStatus" class="apply-rules-status">{{ applyRulesStatus }}</span>
          <button
            class="apply-rules-btn"
            :disabled="applyingRules"
            @click="applyRules"
          >
            {{ applyingRules ? 'Applying...' : 'Apply Rules' }}
          </button>
          <button @click="router.push('/import')">Import CSV</button>
        </div>
      </div>

      <!-- Error -->
      <div v-if="error" class="card error-card">{{ error }}</div>

      <!-- Loading -->
      <div v-else-if="loading" class="card">
        <div class="placeholder">Loading transactions...</div>
      </div>

      <!-- Empty state -->
      <div v-else-if="rows.length === 0" class="card">
        <div class="placeholder">No transactions yet — import a CSV to get started.</div>
      </div>

      <!-- Transaction table -->
      <template v-else>
        <div class="table-wrapper">
          <table class="txn-table">
            <thead>
              <tr>
                <th class="sortable" @click="toggleSort('posting_date')">
                  Date
                  <ChevronUp v-if="sortColumn === 'posting_date' && sortDirection === 'asc'" :size="14" />
                  <ChevronDown v-else-if="sortColumn === 'posting_date' && sortDirection === 'desc'" :size="14" />
                </th>
                <th class="sortable" @click="toggleSort('description')">
                  Description
                  <ChevronUp v-if="sortColumn === 'description' && sortDirection === 'asc'" :size="14" />
                  <ChevronDown v-else-if="sortColumn === 'description' && sortDirection === 'desc'" :size="14" />
                </th>
                <th class="sortable" @click="toggleSort('amount_cents')">
                  Amount
                  <ChevronUp v-if="sortColumn === 'amount_cents' && sortDirection === 'asc'" :size="14" />
                  <ChevronDown v-else-if="sortColumn === 'amount_cents' && sortDirection === 'desc'" :size="14" />
                </th>
                <th class="sortable" @click="toggleSort('transaction_type')">
                  Type
                  <ChevronUp v-if="sortColumn === 'transaction_type' && sortDirection === 'asc'" :size="14" />
                  <ChevronDown v-else-if="sortColumn === 'transaction_type' && sortDirection === 'desc'" :size="14" />
                </th>
                <th class="sortable" @click="toggleSort('category')">
                  Category
                  <ChevronUp v-if="sortColumn === 'category' && sortDirection === 'asc'" :size="14" />
                  <ChevronDown v-else-if="sortColumn === 'category' && sortDirection === 'desc'" :size="14" />
                </th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="row in sortedRows" :key="row.id">
                <td>{{ row.posting_date }}</td>
                <td class="desc-cell">{{ row.description }}</td>
                <td :class="row.amount_cents >= 0 ? 'amount-pos' : 'amount-neg'">
                  {{ formatCents(row.amount_cents) }}
                </td>
                <td>{{ row.transaction_type }}</td>
                <td :class="{ uncategorized: !row.category_id }">
                  {{ categoryName(row.category_id) }}
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </template>
    </main>

  </div>
</template>

<style scoped>
/* ---------- Two-column layout ---------- */
.layout {
  display: flex;
  gap: 1.5rem;
  max-width: none !important; /* override .view max-width */
}

.control-panel {
  width: 280px;
  min-width: 280px;
  background-color: var(--bg-surface);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 1.25rem;
  align-self: flex-start;
  position: sticky;
  top: 1.5rem;
}

.control-panel h3 {
  margin-bottom: 1.25rem;
  color: var(--text-secondary);
}

.content {
  flex: 1;
  min-width: 0; /* prevent flex child overflow */
}

/* ---------- Search ---------- */
.search-input {
  width: 100%;
  padding: 0.5rem 0.75rem;
  background-color: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  color: var(--text-primary);
  font-size: 0.85rem;
  box-sizing: border-box;
}

.search-input:focus {
  border-color: var(--accent);
  outline: none;
}

/* ---------- Filter groups ---------- */
.filter-group {
  margin-bottom: 1.25rem;
}

.filter-label {
  display: block;
  font-size: 0.8rem;
  color: var(--text-muted);
  text-transform: uppercase;
  margin-bottom: 0.4rem;
  font-weight: 500;
}

/* ---------- Multiselect dropdown ---------- */
.multiselect {
  position: relative;
}

.multiselect-toggle {
  width: 100%;
  text-align: left;
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.5rem 0.75rem;
  background-color: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  color: var(--text-secondary);
  font-size: 0.85rem;
  cursor: pointer;
}

.multiselect-toggle:hover {
  border-color: var(--accent);
}

.arrow {
  font-size: 0.65rem;
  color: var(--text-faint);
}

.multiselect-menu {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  right: 0;
  background-color: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  max-height: 220px;
  overflow-y: auto;
  z-index: 10;
}

.multiselect-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.45rem 0.75rem;
  cursor: pointer;
  font-size: 0.85rem;
  color: var(--text-secondary);
}

.multiselect-item:hover {
  background-color: var(--bg-elevated);
}

.multiselect-item input[type="checkbox"] {
  accent-color: var(--accent);
}

.multiselect-empty {
  padding: 0.75rem;
  color: var(--text-muted);
  font-size: 0.85rem;
  text-align: center;
}

/* ---------- Header actions ---------- */
.header-actions {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.apply-rules-btn {
  padding: 0.5rem 1rem;
  background: transparent;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  font-size: 0.85rem;
  cursor: pointer;
  color: var(--text-secondary);
}

.apply-rules-btn:hover {
  border-color: var(--accent);
  color: var(--accent);
}

.apply-rules-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.apply-rules-status {
  font-size: 0.8rem;
  color: var(--text-muted);
}

/* ---------- Rename account ---------- */
.rename-section {
  margin-top: 1.5rem;
  padding-top: 1.25rem;
  border-top: 1px solid var(--border-color);
}

.rename-section h3 {
  margin-bottom: 1.25rem;
  color: var(--text-secondary);
}

.rename-input {
  width: 100%;
  padding: 0.5rem 0.75rem;
  background-color: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  color: var(--text-primary);
  font-size: 0.85rem;
  box-sizing: border-box;
}

.rename-input:focus {
  border-color: var(--accent);
  outline: none;
}

.rename-btn {
  width: 100%;
  margin-top: 0.5rem;
  padding: 0.5rem;
  background-color: var(--accent);
  color: var(--bg-base);
  border: none;
  border-radius: 6px;
  font-size: 0.85rem;
  cursor: pointer;
  font-weight: 500;
}

.rename-btn:hover {
  opacity: 0.9;
}

.rename-status {
  margin-top: 0.5rem;
  font-size: 0.8rem;
  color: var(--accent);
}

/* ---------- Existing styles ---------- */
.header-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1rem;
}

.header-row h2 {
  margin-bottom: 0;
}

.error-card {
  color: var(--danger);
  border-color: var(--danger);
}

.placeholder {
  color: var(--text-faint);
  padding: 3rem;
  text-align: center;
  border: 1px dashed var(--border-color);
  border-radius: 8px;
}

.table-wrapper {
  overflow-x: auto;
  margin-bottom: 1rem;
}

.txn-table {
  width: 100%;
  border-collapse: collapse;
  background-color: var(--bg-elevated);
  border: 1px solid var(--border-color);
  border-radius: 8px;
}

.txn-table th,
.txn-table td {
  padding: 0.6rem 0.8rem;
  text-align: left;
  border-bottom: 1px solid var(--border-subtle);
  font-size: 0.9rem;
}

.txn-table th {
  color: var(--text-muted);
  font-weight: 500;
  font-size: 0.8rem;
  text-transform: uppercase;
}

.txn-table th.sortable {
  cursor: pointer;
  user-select: none;
  display: table-cell;
}

.txn-table th.sortable:hover {
  color: var(--text-primary);
}

/* Inline the icon next to the header text */
.txn-table th.sortable svg {
  display: inline-block;
  vertical-align: middle;
  margin-left: 0.25rem;
  color: var(--accent);
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

.uncategorized {
  color: var(--text-muted);
  font-style: italic;
}

</style>
