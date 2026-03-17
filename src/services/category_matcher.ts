import type { ParsedTransaction, StagedTransaction, CategoryRule } from '@/types'
import { getCategoryRules } from '@/services/getters'


// Check if a text value matches a pattern using the given match type
function textMatches(value: string, pattern: string, matchType: string): boolean {
  const lower = value.toLowerCase()
  const pat = pattern.toLowerCase()

  switch (matchType) {
    case 'exact':       return lower === pat
    case 'contains':    return lower.includes(pat)
    case 'starts_with': return lower.startsWith(pat)
    default:            return false
  }
}


// Check if an amount satisfies the operator comparison (both in cents)
function amountMatches(amount: number, operator: string, target: number): boolean {
  switch (operator) {
    case '<':  return amount < target
    case '>':  return amount > target
    case '=':  return amount === target
    case '<=': return amount <= target
    case '>=': return amount >= target
    default:   return false
  }
}


// Returns true if ALL non-null conditions on the rule match the transaction
// Accepts ParsedTransaction or Transaction (only uses fields the matcher cares about)
export function ruleMatchesTransaction(rule: CategoryRule, tx: Pick<ParsedTransaction, 'details' | 'description' | 'transaction_type' | 'amount_cents'>): boolean {

  // Check details field
  if (rule.details_pattern && rule.details_match_type) {
    if (!textMatches(tx.details, rule.details_pattern, rule.details_match_type)) return false
  }

  // Check description field
  if (rule.description_pattern && rule.description_match_type) {
    if (!textMatches(tx.description, rule.description_pattern, rule.description_match_type)) return false
  }

  // Check type field
  if (rule.type_pattern && rule.type_match_type) {
    if (!textMatches(tx.transaction_type, rule.type_pattern, rule.type_match_type)) return false
  }

  // Check amount
  if (rule.amount_operator && rule.amount_value !== null) {
    if (!amountMatches(tx.amount_cents, rule.amount_operator, rule.amount_value)) return false
  }

  return true
}


export async function applyCategoryRules(
  transactions: ParsedTransaction[],
): Promise<StagedTransaction[]> {

  const staged: StagedTransaction[] = []

  // Rules come back sorted by priority DESC from the getter
  const rules = await getCategoryRules()

  for (const tx of transactions) {

    let category_id: number | null = null
    let rule_matched = false

    // First matching rule wins (highest priority first)
    for (const rule of rules) {
      if (ruleMatchesTransaction(rule, tx)) {
        category_id = rule.category_id
        rule_matched = true
        break
      }
    }

    staged.push({
      ...tx,
      category_id,
      rule_matched,
    })
  }

  return staged
}