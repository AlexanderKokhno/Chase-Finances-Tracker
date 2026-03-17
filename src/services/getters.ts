import { getDatabase } from '@/services/database'
import type { CategoryRule, Category, Account, Transaction } from '@/types'

export async function getCategories(): Promise<Category[]> {
    const db = await getDatabase()
    return await db.select<Category[]>(
        'SELECT id, display_name AS name FROM categories ORDER BY display_name'
    )
}

export async function getCategoryRules(): Promise<CategoryRule[]> {
    const db = await getDatabase()
    return await db.select<CategoryRule[]>(
        'SELECT * FROM category_rules ORDER BY priority DESC'
    )
}

export async function getAccounts(): Promise<Account[]> {
    const db = await getDatabase()
    return await db.select<Account[]>(
        'SELECT id, csv_id, display_name AS name FROM accounts ORDER BY display_name'
    )
}

export async function updateAccountName(id: number, name: string): Promise<void> {
    const db = await getDatabase()
    await db.execute(
        'UPDATE accounts SET display_name = ?1 WHERE id = ?2',
        [name, id]
    )
}

export async function updateTransactionCategory(id: number, categoryId: number | null): Promise<void> {
    const db = await getDatabase()
    await db.execute(
        'UPDATE transactions SET category_id = ?1 WHERE id = ?2',
        [categoryId, id]
    )
}

export async function getTransactions(
    accountIds: number[] = [],
    categoryIds: number[] = []
): Promise<Transaction[]> {
    const db = await getDatabase()
    const conditions: string[] = []
    const params: unknown[] = []

    if (accountIds.length > 0) {
        const placeholders = accountIds.map(() => '?').join(', ')
        conditions.push(`account_id IN (${placeholders})`)
        params.push(...accountIds)
    }

    if (categoryIds.length > 0) {
        const placeholders = categoryIds.map(() => '?').join(', ')
        conditions.push(`category_id IN (${placeholders})`)
        params.push(...categoryIds)
    }

    const where = conditions.length > 0
        ? 'WHERE ' + conditions.join(' AND ')
        : ''

    return await db.select<Transaction[]>(
        `SELECT * FROM transactions ${where} ORDER BY posting_date DESC, id DESC`,
        params
    )
}