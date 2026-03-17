import Database from '@tauri-apps/plugin-sql'
import schema from '@/schema.sql?raw'

let db: Database | null = null

export async function initDatabase(): Promise<Database> {
  if (db) return db

  db = await Database.load('sqlite:chase_tracker.db')

  // Execute each statement from schema.sql
  const statements = schema.split(';').map(s => s.trim()).filter(s => s.length > 0)
  for (const stmt of statements) {
    await db.execute(stmt)
  }

  console.log('Database loaded and schema initialized')

  return db
}

export async function getDatabase(): Promise<Database> {
  if (!db) {
    return initDatabase()
  }
  return db
}
