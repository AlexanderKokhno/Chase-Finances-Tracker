import { getDatabase } from './database'

// Get a single setting by key, returns null if not found
export async function getSetting(key: string): Promise<string | null> {
  const db = await getDatabase()
  const rows = await db.select<{ value: string }[]>(
    'SELECT value FROM app_settings WHERE key = ?',
    [key]
  )
  return rows.length > 0 ? rows[0].value : null
}

// Set a setting (insert or update)
export async function setSetting(key: string, value: string): Promise<void> {
  const db = await getDatabase()
  await db.execute(
    'INSERT OR REPLACE INTO app_settings (key, value) VALUES (?, ?)',
    [key, value]
  )
}

// Get all settings as a key-value object
export async function getAllSettings(): Promise<Record<string, string>> {
  const db = await getDatabase()
  const rows = await db.select<{ key: string; value: string }[]>(
    'SELECT key, value FROM app_settings'
  )
  const result: Record<string, string> = {}
  for (const row of rows) {
    result[row.key] = row.value
  }
  return result
}
