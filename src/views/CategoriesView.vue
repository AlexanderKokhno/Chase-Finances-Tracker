<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { getCategories, getCategoryRules } from '@/services/getters'
import type { Category, CategoryRule } from '@/types'

// ---------- Types ----------

interface CategoryWithRules {
  id: number
  name: string
  rules: CategoryRule[]
}

// ---------- Display state ----------

const groups = ref<CategoryWithRules[]>([])
const allCategories = ref<Category[]>([])
const allRules = ref<CategoryRule[]>([])
const loading = ref(true)
const error = ref('')
const searchQuery = ref('')

// ---------- Form state ----------

const mode = ref<'new' | 'existing'>('existing')
const selectedCategoryId = ref<number | null>(null)
const newCategoryName = ref('')

// Rule fields
const priority = ref(0)
const detailsPattern = ref('')
const detailsMatchType = ref('contains')
const descriptionPattern = ref('')
const descriptionMatchType = ref('contains')
const typePattern = ref('')
const typeMatchType = ref('exact')
const amountOperator = ref('')
const amountValue = ref('')

const submitting = ref(false)
const formError = ref('')

// ---------- Computed ----------

// Filter categories/rules by search query
const filteredGroups = computed(() => {
  const q = searchQuery.value.toLowerCase().trim()
  if (!q) return groups.value
  return groups.value.filter(group => {
    // Match category name
    if (group.name.toLowerCase().includes(q)) return true
    // Match any rule description
    return group.rules.some(rule => describeRule(rule).toLowerCase().includes(q))
  })
})

// The category ID to use — either picked from dropdown, or generated for new
const targetCategoryId = computed(() => {
  if (mode.value === 'existing') return selectedCategoryId.value
  // For new: max existing id + 1
  if (allCategories.value.length === 0) return 1
  const maxId = Math.max(...allCategories.value.map(c => c.id))
  return maxId + 1
})

// ---------- Helpers ----------

function describeRule(rule: CategoryRule): string {
  const parts: string[] = []
  if (rule.details_pattern) {
    parts.push(`Details ${rule.details_match_type} "${rule.details_pattern}"`)
  }
  if (rule.description_pattern) {
    parts.push(`Description ${rule.description_match_type} "${rule.description_pattern}"`)
  }
  if (rule.type_pattern) {
    parts.push(`Type ${rule.type_match_type} "${rule.type_pattern}"`)
  }
  if (rule.amount_operator && rule.amount_value !== null) {
    const dollars = (rule.amount_value / 100).toFixed(2)
    parts.push(`Amount ${rule.amount_operator} $${dollars}`)
  }
  return parts.length > 0 ? parts.join(' & ') : '(no conditions)'
}

// Generate the next rule ID for a given category
// Scheme: category_id * 1000 + sequential number starting at 1
function nextRuleId(categoryId: number): number {
  const existingForCat = allRules.value.filter(r => r.category_id === categoryId)
  if (existingForCat.length === 0) return categoryId * 1000 + 1
  const maxRuleId = Math.max(...existingForCat.map(r => r.id))
  return maxRuleId + 1
}

function resetForm() {
  priority.value = 0
  detailsPattern.value = ''
  detailsMatchType.value = 'contains'
  descriptionPattern.value = ''
  descriptionMatchType.value = 'contains'
  typePattern.value = ''
  typeMatchType.value = 'exact'
  amountOperator.value = ''
  amountValue.value = ''
  formError.value = ''
}

// ---------- Data loading ----------

async function loadData() {
  loading.value = true
  error.value = ''
  try {
    allCategories.value = await getCategories()
    allRules.value = await getCategoryRules()

    const sorted = [...allCategories.value].sort((a, b) => a.id - b.id)
    groups.value = sorted.map(cat => ({
      id: cat.id,
      name: cat.name,
      rules: allRules.value.filter(r => r.category_id === cat.id)
    }))
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

// ---------- Submit ----------

async function addRule() {
  formError.value = ''

  // Validation
  if (mode.value === 'existing' && selectedCategoryId.value === null) {
    formError.value = 'Select a category.'
    return
  }
  if (mode.value === 'new' && newCategoryName.value.trim() === '') {
    formError.value = 'Enter a category name.'
    return
  }

  // At least one condition must be set
  const hasPattern = detailsPattern.value || descriptionPattern.value || typePattern.value
  const hasAmount = amountOperator.value || amountValue.value
  if (!hasPattern && !hasAmount) {
    formError.value = 'Add at least one condition (pattern or amount).'
    return
  }

  // Amount operator and value must both be set if either is
  if (amountOperator.value && !amountValue.value) {
    formError.value = 'Enter an amount value.'
    return
  }
  if (amountValue.value && !amountOperator.value) {
    formError.value = 'Select an amount operator.'
    return
  }

  // Amount value must be a valid number
  if (amountValue.value && isNaN(parseFloat(amountValue.value))) {
    formError.value = 'Amount must be a valid number.'
    return
  }

  const catId = targetCategoryId.value!
  const ruleId = nextRuleId(catId)

  // Build the category array (only needed if new)
  const categories = mode.value === 'new'
    ? [{ id: catId, display_name: newCategoryName.value.trim() }]
    : []

  // Build the rule — null out empty strings
  const rule = {
    id: ruleId,
    category_id: catId,
    priority: priority.value,
    details_pattern: detailsPattern.value || null,
    details_match_type: detailsPattern.value ? detailsMatchType.value : null,
    description_pattern: descriptionPattern.value || null,
    description_match_type: descriptionPattern.value ? descriptionMatchType.value : null,
    type_pattern: typePattern.value || null,
    type_match_type: typePattern.value ? typeMatchType.value : null,
    amount_operator: amountOperator.value || null,
    amount_value: amountValue.value ? Math.round(parseFloat(amountValue.value) * 100) : null,
  }

  submitting.value = true
  try {
    await invoke('import_category_rules', { categories, rules: [rule] })
    resetForm()
    newCategoryName.value = ''
    // Re-pull fresh data
    await loadData()
  } catch (e) {
    formError.value = String(e)
  } finally {
    submitting.value = false
  }
}

// ---------- Delete ----------

async function deleteCategory(categoryId: number) {
  try {
    await invoke('delete_category', { categoryId })
    await loadData()
  } catch (e) {
    error.value = String(e)
  }
}

async function deleteRule(ruleId: number) {
  try {
    await invoke('delete_rule', { ruleId })
    await loadData()
  } catch (e) {
    error.value = String(e)
  }
}

onMounted(loadData)
</script>

<template>
  <div class="view layout">
    <!-- Left control panel -->
    <aside class="control-panel">
      <!-- Search -->
      <div class="panel-card">
        <h3>Search</h3>
        <input
          v-model="searchQuery"
          class="form-input"
          placeholder="Search categories or rules..."
        />
      </div>

      <!-- Add Rule -->
      <div class="panel-card">
      <h3>Add Rule</h3>

      <!-- Mode toggle -->
      <div class="filter-group">
        <label class="filter-label">Category</label>
        <div class="mode-toggle">
          <button
            :class="['toggle-btn', { active: mode === 'existing' }]"
            @click="mode = 'existing'"
          >Existing</button>
          <button
            :class="['toggle-btn', { active: mode === 'new' }]"
            @click="mode = 'new'"
          >New</button>
        </div>
      </div>

      <!-- Existing category dropdown -->
      <div v-if="mode === 'existing'" class="filter-group">
        <select v-model="selectedCategoryId">
          <option :value="null" disabled>Select category...</option>
          <option
            v-for="cat in allCategories"
            :key="cat.id"
            :value="cat.id"
          >{{ cat.name }}</option>
        </select>
      </div>

      <!-- New category name -->
      <div v-else class="filter-group">
        <input
          v-model="newCategoryName"
          class="form-input"
          placeholder="Category name"
        />
      </div>

      <hr class="divider" />

      <!-- Rule fields -->
      <div class="filter-group">
        <label class="filter-label">Priority</label>
        <input v-model.number="priority" type="number" class="form-input" min="0" />
      </div>

      <div class="filter-group">
        <label class="filter-label">Details pattern</label>
        <input v-model="detailsPattern" class="form-input" placeholder="e.g. CHASE CREDIT" />
        <select v-model="detailsMatchType" class="mt-sm">
          <option value="exact">Exact</option>
          <option value="contains">Contains</option>
          <option value="starts_with">Starts with</option>
        </select>
      </div>

      <div class="filter-group">
        <label class="filter-label">Description pattern</label>
        <input v-model="descriptionPattern" class="form-input" placeholder="e.g. WALMART" />
        <select v-model="descriptionMatchType" class="mt-sm">
          <option value="exact">Exact</option>
          <option value="contains">Contains</option>
          <option value="starts_with">Starts with</option>
        </select>
      </div>

      <div class="filter-group">
        <label class="filter-label">Type pattern</label>
        <input v-model="typePattern" class="form-input" placeholder="e.g. DEBIT" />
        <select v-model="typeMatchType" class="mt-sm">
          <option value="exact">Exact</option>
          <option value="contains">Contains</option>
          <option value="starts_with">Starts with</option>
        </select>
      </div>

      <div class="filter-group">
        <label class="filter-label">Amount condition</label>
        <div class="amount-row">
          <select v-model="amountOperator" class="amount-op">
            <option value="">None</option>
            <option value="<">&lt;</option>
            <option value=">">&gt;</option>
            <option value="=">=</option>
            <option value="<=">&lt;=</option>
            <option value=">=">&gt;=</option>
          </select>
          <input
            v-model="amountValue"
            class="form-input"
            placeholder="$0.00"
            :disabled="!amountOperator"
          />
        </div>
      </div>

      <!-- Error message -->
      <div v-if="formError" class="form-error">{{ formError }}</div>

      <!-- Submit -->
      <button
        class="submit-btn"
        :disabled="submitting"
        @click="addRule"
      >
        {{ submitting ? 'Adding...' : 'Add Rule' }}
      </button>

      </div>
    </aside>

    <!-- Right content area -->
    <main class="content">
      <h2>Categories &amp; Rules</h2>

      <!-- Error -->
      <div v-if="error" class="card error-card">{{ error }}</div>

      <!-- Loading -->
      <div v-else-if="loading" class="card">
        <div class="placeholder">Loading categories...</div>
      </div>

      <!-- Empty state -->
      <div v-else-if="filteredGroups.length === 0" class="card">
        <div class="placeholder">No categories yet — add one using the panel on the left.</div>
      </div>

      <!-- Category + rules list -->
      <div v-else class="cat-list">
        <div v-for="group in filteredGroups" :key="group.id" class="cat-group">
          <div class="cat-name">
            <span>{{ group.name }}</span>
            <span class="delete-icon delete-category" @click="deleteCategory(group.id)">&#128465;</span>
          </div>

          <div v-if="group.rules.length === 0" class="rule-row no-rules">
            No rules
          </div>
          <div v-for="rule in group.rules" :key="rule.id" class="rule-row">
            <span class="rule-priority">P{{ rule.priority }}</span>
            <span class="rule-desc">{{ describeRule(rule) }}</span>
            <span class="delete-icon" @click="deleteRule(rule.id)">&#128465;</span>
          </div>
        </div>
      </div>
    </main>
  </div>
</template>

<style scoped>
/* ---------- Two-column layout (matches TransactionsView) ---------- */
.layout {
  display: flex;
  gap: 1.5rem;
  max-width: none !important;
}

.control-panel {
  width: 280px;
  min-width: 280px;
  display: flex;
  flex-direction: column;
  gap: 1rem;
  align-self: flex-start;
  position: sticky;
  top: 1.5rem;
  max-height: calc(100vh - 3rem);
  overflow-y: auto;
}

.panel-card {
  background-color: var(--bg-surface);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 1.25rem;
}

.panel-card h3 {
  margin-bottom: 1.25rem;
  color: var(--text-secondary);
}

.content {
  flex: 1;
  min-width: 0;
}

/* ---------- Mode toggle ---------- */
.mode-toggle {
  display: flex;
  gap: 0;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  overflow: hidden;
}

.toggle-btn {
  flex: 1;
  padding: 0.4rem 0.5rem;
  font-size: 0.8rem;
  background: var(--bg-input);
  color: var(--text-faint);
  border: none;
  cursor: pointer;
}

.toggle-btn.active {
  background: var(--accent);
  color: #111;
  font-weight: 600;
}

/* ---------- Form elements ---------- */
.filter-group {
  margin-bottom: 1rem;
}

.filter-label {
  display: block;
  font-size: 0.8rem;
  color: var(--text-muted);
  text-transform: uppercase;
  margin-bottom: 0.4rem;
  font-weight: 500;
}

.form-input {
  width: 100%;
  padding: 0.45rem 0.6rem;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  font-size: 0.85rem;
  box-sizing: border-box;
}

input.form-input {
  background: var(--bg-input);
  color: var(--text-primary);
}

.form-input:focus {
  outline: none;
  border-color: var(--accent);
}

.form-input:disabled {
  opacity: 0.4;
}

.mt-sm {
  margin-top: 0.35rem;
}

.amount-row {
  display: flex;
  gap: 0.4rem;
}

.amount-op {
  width: 85px;
  min-width: 85px;
  flex: none;
}

.divider {
  border: none;
  border-top: 1px solid var(--border-color);
  margin: 0.75rem 0;
}

.form-error {
  color: var(--danger);
  font-size: 0.8rem;
  margin-bottom: 0.75rem;
}

.submit-btn {
  width: 100%;
  padding: 0.55rem;
  background: var(--accent);
  color: #111;
  border: none;
  border-radius: 6px;
  font-weight: 600;
  font-size: 0.85rem;
  cursor: pointer;
}

.submit-btn:hover {
  background: var(--accent-hover);
}

.submit-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}


/* ---------- Main content ---------- */
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

.cat-list {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.cat-group {
  background-color: var(--bg-elevated);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 0.75rem 1rem;
}

.cat-name {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-weight: 600;
  font-size: 1rem;
  color: var(--text-primary);
  margin-bottom: 0.35rem;
}

.rule-row {
  padding: 0.3rem 0 0.3rem 1.25rem;
  font-size: 0.85rem;
  color: var(--text-secondary);
  display: flex;
  gap: 0.6rem;
  align-items: baseline;
}

.rule-priority {
  color: var(--accent);
  font-weight: 500;
  min-width: 2rem;
}

.rule-desc {
  color: var(--text-muted);
}

.delete-icon {
  margin-left: auto;
  color: var(--danger);
  cursor: pointer;
  opacity: 0;
  transition: opacity 0.15s;
  font-size: 1rem;
}

.delete-category {
  background-color: var(--danger);
  color: #fff;
  border-radius: 4px;
  padding: 0.15rem 0.35rem;
  font-size: 0.85rem;
  line-height: 1;
}

.cat-name:hover .delete-icon,
.rule-row:hover .delete-icon {
  opacity: 1;
}

.no-rules {
  color: var(--text-muted);
  font-style: italic;
}
</style>
