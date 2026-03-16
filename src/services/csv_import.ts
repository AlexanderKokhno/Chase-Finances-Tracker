import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import { applyCategoryRules } from '@/services/category_matcher'
import type { CsvParseData, StagedTransaction } from '@/types'



// Opens file dialog, sends path to Rust for parsing, applies category rules
// Returns the staged transactions + account ID, or null if user cancelled
export async function openAndParseCsv(): Promise<{
  account_id: string
  transactions: StagedTransaction[]
  skipped_rows: string[]
} | null> {

  // 1. Open file picker
  const selected = await open({
    filters: [{ name: 'CSV Files', extensions: ['csv', 'CSV'] }],
    multiple: false,
  })

  if (!selected) return null // user cancelled

  // 2. Send file path to Rust for parsing
  const parsed = await invoke<CsvParseData>('parse_chase_csv', {
    filePath: selected,
  })

  // 4. Apply rules to the parsed transactions
  const transactions = await applyCategoryRules(parsed.transactions)



  return {
    account_id: parsed.account_id,
    transactions,
    skipped_rows: parsed.skipped_rows,
  }
}


// Sends staged transactions to Rust backend for writing to the database
export async function confirmImport(
  account_id: string,
  transactions: StagedTransaction[]
): Promise<void> {
  await invoke('import_transactions', {
    accountId: account_id,
    transactions,
  })
}
