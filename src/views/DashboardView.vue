<script setup lang="ts">
import { ref, computed, watch, onMounted, onBeforeUnmount } from 'vue'
import VChart from 'vue-echarts'
import { use } from 'echarts/core'
import { LineChart, BarChart, CandlestickChart } from 'echarts/charts'
import { GridComponent, TooltipComponent, LegendComponent } from 'echarts/components'
import { CanvasRenderer } from 'echarts/renderers'
import { getTransactions, getAccounts, getCategories } from '../services/getters'
import type { Transaction, Account } from '../types'

use([LineChart, BarChart, CandlestickChart, GridComponent, TooltipComponent, LegendComponent, CanvasRenderer])

// --- Raw data (loaded once) ---
const allAccounts = ref<Account[]>([])
const allTransactions = ref<Transaction[]>([])
const transferCategoryId = ref<number | null>(null)
const payrollCategoryId = ref<number | null>(null)

// --- Category options for dropdown ---
interface FilterOption { id: number; label: string }
const categoryOptions = ref<FilterOption[]>([])
const selectedCategoryIds = ref<number[]>([])   // -1 = "Not Categorized"
const categoryDropdownOpen = ref(false)

// --- Filter state ---
const selectedAccountIds = ref<Set<number>>(new Set())
const includeTransfers = ref(false)
const syncScales = ref(true)
const dateFrom = ref('')
const dateTo = ref('')
const dateMin = ref('')   // earliest date in data (for clamping)
const dateMax = ref('')   // latest date in data (for clamping)

// --- Chart options (rebuilt when filters change) ---
const balanceOption = ref({})
const flowOption = ref({})
const candleOption = ref({})
const payrollOption = ref({})

// --- Helpers ---

function toMonthKey(postingDate: string): string {
  const [month, , year] = postingDate.split('/')
  return `${year}-${month}`
}

function formatMonthLabel(key: string): string {
  const [year, month] = key.split('-')
  const date = new Date(Number(year), Number(month) - 1)
  return date.toLocaleDateString('en-US', { month: 'short', year: 'numeric' })
}

function makeDarkAxis(months: string[]) {
  return {
    grid: { left: 80, right: 30, top: 50, bottom: 40 },
    xAxis: {
      type: 'category',
      data: months,
      axisLabel: { color: '#888' },
      axisLine: { lineStyle: { color: '#444' } }
    },
    yAxis: {
      type: 'value',
      axisLabel: {
        color: '#888',
        formatter: (val: number) => `$${(val / 1000).toFixed(0)}k`
      },
      splitLine: { lineStyle: { color: '#333' } }
    },
    tooltip: {
      trigger: 'axis',
      formatter: (params: any) => {
        let html = `<b>${params[0].name}</b>`
        for (const p of params) {
          const val = p.value.toLocaleString('en-US', { minimumFractionDigits: 2 })
          html += `<br/>${p.marker} ${p.seriesName}: $${val}`
        }
        return html
      }
    },
    legend: {
      top: 0,
      right: 0,
      textStyle: { color: '#ccc' }
    }
  }
}

// --- Toggle helpers ---

function toggleAccount(id: number) {
  const s = new Set(selectedAccountIds.value)
  if (s.has(id)) s.delete(id)
  else s.add(id)
  selectedAccountIds.value = s
}

function selectAll() {
  selectedAccountIds.value = new Set(allAccounts.value.map(a => a.id))
}

function selectNone() {
  selectedAccountIds.value = new Set()
}

function toggleCategory(id: number) {
  const idx = selectedCategoryIds.value.indexOf(id)
  if (idx === -1) {
    selectedCategoryIds.value.push(id)
  } else {
    selectedCategoryIds.value.splice(idx, 1)
  }
}

function selectAllCategories() {
  selectedCategoryIds.value = categoryOptions.value.map(o => o.id)
}

function selectNoCategories() {
  selectedCategoryIds.value = []
}

// --- Custom calendar picker state ---
const calendarOpen = ref<'from' | 'to' | null>(null)
const calendarViewYear = ref(new Date().getFullYear())
const calendarViewMonth = ref(new Date().getMonth()) // 0-based

const DAY_NAMES = ['Su', 'Mo', 'Tu', 'We', 'Th', 'Fr', 'Sa']

// Label for the month/year header in the calendar
const calendarMonthLabel = computed(() => {
  const d = new Date(calendarViewYear.value, calendarViewMonth.value)
  return d.toLocaleDateString('en-US', { month: 'long', year: 'numeric' })
})

// Build the 6-row x 7-col grid of day cells for the current view month
const calendarDays = computed(() => {
  const year = calendarViewYear.value
  const month = calendarViewMonth.value
  const firstDay = new Date(year, month, 1).getDay() // 0=Sun
  const daysInMonth = new Date(year, month + 1, 0).getDate()

  const cells: (number | null)[] = []
  // Leading blanks
  for (let i = 0; i < firstDay; i++) cells.push(null)
  // Actual days
  for (let d = 1; d <= daysInMonth; d++) cells.push(d)
  return cells
})

function openCalendar(which: 'from' | 'to') {
  // Set the calendar view to the month of the currently selected date
  const current = which === 'from' ? dateFrom.value : dateTo.value
  if (current) {
    const d = new Date(current + 'T00:00:00')
    calendarViewYear.value = d.getFullYear()
    calendarViewMonth.value = d.getMonth()
  }
  calendarOpen.value = which
}

function calendarPrevMonth() {
  if (calendarViewMonth.value === 0) {
    calendarViewMonth.value = 11
    calendarViewYear.value--
  } else {
    calendarViewMonth.value--
  }
}

function calendarNextMonth() {
  if (calendarViewMonth.value === 11) {
    calendarViewMonth.value = 0
    calendarViewYear.value++
  } else {
    calendarViewMonth.value++
  }
}

// Format a Date to YYYY-MM-DD string
function toISODate(d: Date): string {
  const y = d.getFullYear()
  const m = String(d.getMonth() + 1).padStart(2, '0')
  const day = String(d.getDate()).padStart(2, '0')
  return `${y}-${m}-${day}`
}

function selectCalendarDay(day: number) {
  const picked = toISODate(new Date(calendarViewYear.value, calendarViewMonth.value, day))

  if (calendarOpen.value === 'from') {
    // Clamp to min/max and don't exceed "to"
    if (dateMin.value && picked < dateMin.value) { dateFrom.value = dateMin.value }
    else if (dateMax.value && picked > dateMax.value) { dateFrom.value = dateMax.value }
    else if (dateTo.value && picked > dateTo.value) { dateFrom.value = dateTo.value }
    else { dateFrom.value = picked }
  } else {
    // Clamp to min/max and don't go before "from"
    if (dateMin.value && picked < dateMin.value) { dateTo.value = dateMin.value }
    else if (dateMax.value && picked > dateMax.value) { dateTo.value = dateMax.value }
    else if (dateFrom.value && picked < dateFrom.value) { dateTo.value = dateFrom.value }
    else { dateTo.value = picked }
  }

  calendarOpen.value = null
}

// Is a given day in the current view month outside the allowed range?
function isDayDisabled(day: number): boolean {
  const d = toISODate(new Date(calendarViewYear.value, calendarViewMonth.value, day))
  if (dateMin.value && d < dateMin.value) return true
  if (dateMax.value && d > dateMax.value) return true
  return false
}

// Is a given day the currently selected date for the active picker?
function isDaySelected(day: number): boolean {
  const d = toISODate(new Date(calendarViewYear.value, calendarViewMonth.value, day))
  if (calendarOpen.value === 'from') return d === dateFrom.value
  if (calendarOpen.value === 'to') return d === dateTo.value
  return false
}

// Format YYYY-MM-DD for display as "MMM DD, YYYY"
function formatDateLabel(isoDate: string): string {
  if (!isoDate) return '—'
  const d = new Date(isoDate + 'T00:00:00')
  return d.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' })
}

function categorySelectionLabel(): string {
  const count = selectedCategoryIds.value.length
  const total = categoryOptions.value.length
  if (count === 0) return 'All categories'
  if (count === total) return 'All categories'
  if (count === 1) {
    const match = categoryOptions.value.find(o => o.id === selectedCategoryIds.value[0])
    return match ? match.label : '1 selected'
  }
  return `${count} selected`
}

// Close dropdowns when clicking outside
function onClickOutside(event: MouseEvent) {
  const target = event.target as HTMLElement
  if (!target.closest('.multiselect')) {
    categoryDropdownOpen.value = false
  }
  if (!target.closest('.datepicker')) {
    calendarOpen.value = null
  }
}

onMounted(() => {
  document.addEventListener('click', onClickOutside)
})

onBeforeUnmount(() => {
  document.removeEventListener('click', onClickOutside)
})

// --- Filter transactions based on current selections ---

// Parse MM/DD/YYYY to a Date for comparison
function parsePostingDate(dateStr: string): Date {
  const [month, day, year] = dateStr.split('/')
  return new Date(Number(year), Number(month) - 1, Number(day))
}

function getFilteredTransactions(): Transaction[] {
  // Pre-parse date range bounds (input type="date" gives YYYY-MM-DD)
  const fromDate = dateFrom.value ? new Date(dateFrom.value + 'T00:00:00') : null
  const toDate = dateTo.value ? new Date(dateTo.value + 'T23:59:59') : null

  // Are categories actively filtered? (empty = show all)
  const catFilterActive = selectedCategoryIds.value.length > 0
    && selectedCategoryIds.value.length < categoryOptions.value.length

  return allTransactions.value.filter(tx => {
    if (!selectedAccountIds.value.has(tx.account_id)) return false
    if (!includeTransfers.value && tx.category_id === transferCategoryId.value) return false

    // Date range filter
    if (fromDate || toDate) {
      const txDate = parsePostingDate(tx.posting_date)
      if (fromDate && txDate < fromDate) return false
      if (toDate && txDate > toDate) return false
    }

    // Category filter
    if (catFilterActive) {
      // -1 represents "Not Categorized" (null category_id)
      if (tx.category_id === null) {
        if (!selectedCategoryIds.value.includes(-1)) return false
      } else {
        if (!selectedCategoryIds.value.includes(tx.category_id)) return false
      }
    }

    return true
  })
}

// --- Build all charts from filtered data ---

function buildCharts() {
  const filtered = getFilteredTransactions()
  if (filtered.length === 0) {
    balanceOption.value = {}
    flowOption.value = {}
    candleOption.value = {}
    payrollOption.value = {}
    return
  }

  // Sorted by date
  const sorted = [...filtered].sort((a, b) => a.posting_date.localeCompare(b.posting_date))

  // Collect all months
  const allMonthKeys = new Set<string>()
  for (const tx of sorted) {
    allMonthKeys.add(toMonthKey(tx.posting_date))
  }
  const sortedMonths = [...allMonthKeys].sort()
  const monthLabels = sortedMonths.map(formatMonthLabel)

  // ---- Balance chart ----
  // Sum the last balance of each selected account per month
  // For each account, get the last balance per month, then carry forward
  const accountBalanceByMonth = new Map<number, Map<string, number>>()

  for (const tx of sorted) {
    if (!selectedAccountIds.value.has(tx.account_id)) continue
    if (!accountBalanceByMonth.has(tx.account_id)) {
      accountBalanceByMonth.set(tx.account_id, new Map())
    }
    accountBalanceByMonth.get(tx.account_id)!.set(
      toMonthKey(tx.posting_date),
      tx.balance_cents / 100
    )
  }

  // For each account, forward-fill then sum across accounts per month
  const combinedBalance = sortedMonths.map(() => 0)
  for (const [, monthMap] of accountBalanceByMonth) {
    let carry: number | null = null
    sortedMonths.forEach((m, i) => {
      if (monthMap.has(m)) carry = monthMap.get(m)!
      if (carry !== null) combinedBalance[i] += carry
    })
  }

  balanceOption.value = {
    ...makeDarkAxis(monthLabels),
    series: [{
      name: 'Combined Balance',
      type: 'line',
      smooth: true,
      data: combinedBalance,
      lineStyle: { color: '#22c55e', width: 2 },
      itemStyle: { color: '#22c55e' }
    }]
  }

  // ---- Net & Volume chart ----
  const monthlyNet = new Map<string, number>()
  const monthlyVolume = new Map<string, number>()

  for (const tx of filtered) {
    const key = toMonthKey(tx.posting_date)
    monthlyNet.set(key, (monthlyNet.get(key) || 0) + tx.amount_cents)
    monthlyVolume.set(key, (monthlyVolume.get(key) || 0) + Math.abs(tx.amount_cents))
  }

  flowOption.value = {
    ...makeDarkAxis(monthLabels),
    series: [
      {
        name: 'Net',
        type: 'line',
        smooth: true,
        data: sortedMonths.map(m => (monthlyNet.get(m) ?? 0) / 100),
        lineStyle: { color: '#3b82f6', width: 2 },
        itemStyle: { color: '#3b82f6' }
      },
      {
        name: 'Volume',
        type: 'bar',
        data: sortedMonths.map(m => (monthlyVolume.get(m) ?? 0) / 100),
        itemStyle: { color: '#3b82f6', opacity: 0.3 }
      }
    ]
  }

  // ---- Candlestick chart ----
  // Track each account's latest balance, sum them at each transaction point
  const monthCombinedBalances = new Map<string, number[]>()
  const runningBalance = new Map<number, number>()

  for (const tx of sorted) {
    if (!selectedAccountIds.value.has(tx.account_id)) continue
    runningBalance.set(tx.account_id, tx.balance_cents / 100)

    // Sum all accounts' current balances
    let combined = 0
    for (const bal of runningBalance.values()) combined += bal

    const key = toMonthKey(tx.posting_date)
    const list = monthCombinedBalances.get(key) || []
    list.push(combined)
    monthCombinedBalances.set(key, list)
  }

  let lastClose: number | null = null
  const candleData = sortedMonths.map(m => {
    const balances = monthCombinedBalances.get(m)
    if (!balances) {
      if (lastClose === null) return '-'
      return [lastClose, lastClose, lastClose, lastClose]
    }
    const open = balances[0]
    const close = balances[balances.length - 1]
    const low = Math.min(...balances)
    const high = Math.max(...balances)
    lastClose = close
    return [open, close, low, high]
  })

  candleOption.value = {
    grid: { left: 80, right: 30, top: 50, bottom: 40 },
    xAxis: {
      type: 'category',
      data: monthLabels,
      axisLabel: { color: '#888' },
      axisLine: { lineStyle: { color: '#444' } }
    },
    yAxis: {
      type: 'value',
      axisLabel: {
        color: '#888',
        formatter: (val: number) => `$${(val / 1000).toFixed(0)}k`
      },
      splitLine: { lineStyle: { color: '#333' } }
    },
    tooltip: {
      trigger: 'axis',
      formatter: (params: any) => {
        let html = `<b>${params[0].name}</b>`
        for (const p of params) {
          if (!p.data || p.data === '-') continue
          html += `<br/>${p.marker} ${p.seriesName}`
          html += `<br/>  Open: $${p.data[1].toLocaleString()}`
          html += `<br/>  Close: $${p.data[2].toLocaleString()}`
          html += `<br/>  Low: $${p.data[3].toLocaleString()}`
          html += `<br/>  High: $${p.data[4].toLocaleString()}`
        }
        return html
      }
    },
    legend: { top: 0, right: 0, textStyle: { color: '#ccc' } },
    series: [{
      name: 'Balance Range',
      type: 'candlestick',
      data: candleData,
      itemStyle: {
        color: '#22c55e',
        color0: '#ef4444',
        borderColor: '#22c55e',
        borderColor0: '#ef4444'
      }
    }]
  }

  // ---- Payroll chart ----
  // Only transactions matching the Payroll category
  const payrollTxs = filtered.filter(tx => tx.category_id === payrollCategoryId.value)
  const monthlyPayroll = new Map<string, number>()
  for (const tx of payrollTxs) {
    const key = toMonthKey(tx.posting_date)
    monthlyPayroll.set(key, (monthlyPayroll.get(key) || 0) + tx.amount_cents)
  }

  const payrollData = sortedMonths.map(m => (monthlyPayroll.get(m) ?? 0) / 100)

  payrollOption.value = {
    ...makeDarkAxis(monthLabels),
    series: [{
      name: 'Payroll',
      type: 'line',
      smooth: true,
      data: payrollData,
      lineStyle: { color: '#f59e0b', width: 2 },
      itemStyle: { color: '#f59e0b' }
    }]
  }

  // ---- Sync scales across all charts ----
  if (syncScales.value) {
    // Collect all y-values from every chart
    const allValues: number[] = []

    // Balance line
    allValues.push(...combinedBalance)

    // Net & Volume
    for (const m of sortedMonths) {
      allValues.push((monthlyNet.get(m) ?? 0) / 100)
      allValues.push((monthlyVolume.get(m) ?? 0) / 100)
    }

    // Candlestick (each entry is [open, close, low, high])
    for (const d of candleData) {
      if (d === '-' || !Array.isArray(d)) continue
      allValues.push(d[0], d[1], d[2], d[3])
    }

    // Payroll
    allValues.push(...payrollData)

    if (allValues.length > 0) {
      const globalMin = Math.min(...allValues)
      const globalMax = Math.max(...allValues)
      // Add 5% padding
      const padding = (globalMax - globalMin) * 0.05 || 1
      const yMin = Math.floor(globalMin - padding)
      const yMax = Math.ceil(globalMax + padding)

      const applyScale = (opt: any) => {
        if (opt && opt.yAxis) {
          opt.yAxis.min = yMin
          opt.yAxis.max = yMax
        }
      }

      applyScale(balanceOption.value)
      applyScale(flowOption.value)
      applyScale(candleOption.value)
      applyScale(payrollOption.value)
    }
  }
}

// --- Load data once, then rebuild charts on filter changes ---

onMounted(async () => {
  const [accounts, transactions, categories] = await Promise.all([
    getAccounts(),
    getTransactions(),
    getCategories()
  ])

  allAccounts.value = accounts
  allTransactions.value = transactions

  // Find category ids by name
  const transferCat = categories.find(c => c.name.toLowerCase() === 'transfers')
  if (transferCat) transferCategoryId.value = transferCat.id
  const payrollCat = categories.find(c => c.name.toLowerCase() === 'payroll')
  if (payrollCat) payrollCategoryId.value = payrollCat.id

  // Build category dropdown options (-1 = Not Categorized)
  categoryOptions.value = [
    { id: -1, label: 'Not Categorized' },
    ...categories.map(c => ({ id: c.id, label: c.name }))
  ]

  // Auto-fill date range from the data's actual min/max posting_date
  if (transactions.length > 0) {
    const dates = transactions.map(tx => parsePostingDate(tx.posting_date))
    const minDate = new Date(Math.min(...dates.map(d => d.getTime())))
    const maxDate = new Date(Math.max(...dates.map(d => d.getTime())))
    // Format as YYYY-MM-DD for the date input
    const toISO = (d: Date) => d.toISOString().split('T')[0]
    dateMin.value = toISO(minDate)
    dateMax.value = toISO(maxDate)
    dateFrom.value = dateMin.value
    dateTo.value = dateMax.value
  }

  // Select all accounts by default
  selectedAccountIds.value = new Set(accounts.map(a => a.id))
})

// Rebuild charts whenever filters or data change
watch(
  [selectedAccountIds, includeTransfers, syncScales, allTransactions, dateFrom, dateTo, selectedCategoryIds],
  () => buildCharts(),
  { deep: true }
)
</script>

<template>
  <div class="view">
    <h2>Dashboard</h2>

    <div class="dashboard-layout">
      <aside class="sidebar">
        <h3>Accounts</h3>
        <div class="filter-actions">
          <button @click="selectAll">All</button>
          <button @click="selectNone">None</button>
        </div>
        <label
          v-for="account in allAccounts"
          :key="account.id"
          class="filter-checkbox"
        >
          <input
            type="checkbox"
            :checked="selectedAccountIds.has(account.id)"
            @change="toggleAccount(account.id)"
          />
          {{ account.name || account.csv_id }}
        </label>

        <hr class="divider" />

        <h3>Date Range</h3>
        <div class="date-range">
          <div class="datepicker" @click.stop>
            <label class="date-label">From</label>
            <button class="datepicker-toggle" @click="openCalendar('from')">
              {{ formatDateLabel(dateFrom) }}
            </button>
          </div>
          <div class="datepicker" @click.stop>
            <label class="date-label">To</label>
            <button class="datepicker-toggle" @click="openCalendar('to')">
              {{ formatDateLabel(dateTo) }}
            </button>
          </div>
          <button class="reset-dates" @click="dateFrom = dateMin; dateTo = dateMax">Reset</button>
        </div>

        <!-- Custom calendar popup -->
        <div v-if="calendarOpen" class="calendar-popup" @click.stop>
          <div class="calendar-header">
            <button class="cal-nav" @click="calendarPrevMonth">&lt;</button>
            <span class="cal-title">{{ calendarMonthLabel }}</span>
            <button class="cal-nav" @click="calendarNextMonth">&gt;</button>
          </div>
          <div class="calendar-grid">
            <div v-for="name in DAY_NAMES" :key="name" class="cal-day-name">{{ name }}</div>
            <div
              v-for="(day, i) in calendarDays"
              :key="i"
              class="cal-cell"
              :class="{
                'cal-empty': day === null,
                'cal-disabled': day !== null && isDayDisabled(day),
                'cal-selected': day !== null && isDaySelected(day)
              }"
              @click="day !== null && !isDayDisabled(day) && selectCalendarDay(day)"
            >
              {{ day ?? '' }}
            </div>
          </div>
        </div>

        <hr class="divider" />

        <h3>Categories</h3>
        <div class="multiselect" @click.stop>
          <button
            class="multiselect-toggle"
            @click="categoryDropdownOpen = !categoryDropdownOpen"
          >
            {{ categorySelectionLabel() }}
            <span class="arrow">{{ categoryDropdownOpen ? '▲' : '▼' }}</span>
          </button>
          <div v-if="categoryDropdownOpen" class="multiselect-menu">
            <div class="filter-actions" style="margin-bottom: 0.25rem;">
              <button @click="selectAllCategories">All</button>
              <button @click="selectNoCategories">None</button>
            </div>
            <label
              v-for="opt in categoryOptions"
              :key="opt.id"
              class="multiselect-item"
            >
              <input
                type="checkbox"
                :checked="selectedCategoryIds.includes(opt.id)"
                @change="toggleCategory(opt.id)"
              />
              {{ opt.label }}
            </label>
          </div>
        </div>

        <hr class="divider" />

        <label class="filter-checkbox">
          <input type="checkbox" v-model="includeTransfers" />
          Include Transfers
        </label>

        <label class="filter-checkbox">
          <input type="checkbox" v-model="syncScales" />
          Sync Chart Scales
        </label>
      </aside>

      <section class="card">
        <h3>Balance Over Time</h3>
        <v-chart v-if="Object.keys(balanceOption).length" :option="balanceOption" class="chart" autoresize />
        <div v-else class="placeholder">No data</div>
      </section>

      <section class="card">
        <h3>Net & Volume Over Time</h3>
        <v-chart v-if="Object.keys(flowOption).length" :option="flowOption" class="chart" autoresize />
        <div v-else class="placeholder">No data</div>
      </section>

      <section class="card">
        <h3>Balance Range</h3>
        <v-chart v-if="Object.keys(candleOption).length" :option="candleOption" class="chart" autoresize />
        <div v-else class="placeholder">No data</div>
      </section>

      <section class="card">
        <h3>Payroll</h3>
        <v-chart v-if="Object.keys(payrollOption).length" :option="payrollOption" class="chart" autoresize />
        <div v-else class="placeholder">No data</div>
      </section>
    </div>
  </div>
</template>

<style scoped>
.view {
  max-width: none;
  margin: 0;
  padding: 1rem;
}

.dashboard-layout {
  display: grid;
  grid-template-columns: 1fr 2fr 2fr;
  gap: 1rem;
  height: calc(100% - 3rem);
}

.sidebar {
  grid-row: 1 / 3;
  position: relative;
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 1rem;
  overflow: visible;
}

.sidebar h3 {
  margin: 0 0 0.5rem;
  color: var(--text-secondary);
  font-size: 0.9rem;
}

.filter-actions {
  display: flex;
  gap: 0.5rem;
  margin-bottom: 0.75rem;
}

.filter-actions button {
  padding: 0.2rem 0.6rem;
  font-size: 0.75rem;
  background: var(--bg-input);
  color: var(--text-secondary);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  cursor: pointer;
}

.filter-actions button:hover {
  background: var(--bg-elevated);
}

.filter-checkbox {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  color: var(--text-secondary);
  font-size: 0.85rem;
  padding: 0.25rem 0;
  cursor: pointer;
}

.divider {
  border: none;
  border-top: 1px solid var(--border-color);
  margin: 0.75rem 0;
}

.card {
  min-width: 0;
  overflow: hidden;
}

.chart {
  height: 300px;
  width: 100%;
}

.placeholder {
  color: var(--text-faint);
  padding: 3rem;
  text-align: center;
  border: 1px dashed var(--border-color);
  border-radius: 8px;
}

/* ---------- Date range & calendar ---------- */
.date-range {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}

.date-label {
  color: var(--text-secondary);
  font-size: 0.8rem;
  margin-bottom: 0.1rem;
}

.datepicker-toggle {
  width: 100%;
  text-align: left;
  padding: 0.35rem 0.5rem;
  font-size: 0.85rem;
  background: var(--bg-input);
  color: var(--text-primary);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  cursor: pointer;
}

.datepicker-toggle:hover {
  border-color: var(--accent);
}

.reset-dates {
  margin-top: 0.25rem;
  padding: 0.2rem 0.6rem;
  font-size: 0.75rem;
  background: var(--bg-input);
  color: var(--text-secondary);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  cursor: pointer;
  align-self: flex-start;
}

.reset-dates:hover {
  background: var(--bg-elevated);
}

.calendar-popup {
  position: absolute;
  z-index: 20;
  margin-top: 0.25rem;
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 0.5rem;
  width: 220px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
}

.calendar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 0.4rem;
}

.cal-nav {
  background: none;
  border: none;
  color: var(--text-secondary);
  font-size: 1rem;
  cursor: pointer;
  padding: 0.15rem 0.4rem;
  border-radius: 4px;
}

.cal-nav:hover {
  background: var(--bg-elevated);
}

.cal-title {
  color: var(--text-primary);
  font-size: 0.85rem;
  font-weight: 600;
}

.calendar-grid {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 1px;
  text-align: center;
}

.cal-day-name {
  font-size: 0.7rem;
  color: var(--text-faint);
  padding: 0.2rem 0;
}

.cal-cell {
  font-size: 0.8rem;
  padding: 0.3rem 0;
  border-radius: 4px;
  cursor: pointer;
  color: var(--text-secondary);
}

.cal-cell:hover:not(.cal-empty):not(.cal-disabled) {
  background: var(--bg-elevated);
}

.cal-empty {
  cursor: default;
}

.cal-disabled {
  color: var(--text-faint);
  opacity: 0.35;
  cursor: not-allowed;
}

.cal-selected {
  background: var(--accent);
  color: #fff;
  font-weight: 600;
}

.cal-selected:hover {
  background: var(--accent);
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
  padding: 0.35rem 0.5rem;
  font-size: 0.85rem;
  background: var(--bg-input);
  color: var(--text-secondary);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  cursor: pointer;
}

.multiselect-toggle:hover {
  border-color: var(--accent);
}

.multiselect-toggle .arrow {
  font-size: 0.65rem;
  color: var(--text-faint);
}

.multiselect-menu {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  right: 0;
  max-height: 200px;
  overflow-y: auto;
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  padding: 0.25rem 0;
  z-index: 10;
}

.multiselect-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.3rem 0.6rem;
  font-size: 0.85rem;
  cursor: pointer;
  color: var(--text-secondary);
}

.multiselect-item:hover {
  background-color: var(--bg-elevated);
}

.multiselect-item input[type="checkbox"] {
  accent-color: var(--accent);
}
</style>
