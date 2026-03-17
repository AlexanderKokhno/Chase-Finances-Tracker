
export interface ParsedTransaction { // Original
  details: string;
  posting_date: string;
  description: string;
  amount_cents: number;
  transaction_type: string;
  balance_cents: number;
  check_or_slip_num: string;
}


export interface CsvParseData {
  account_id: string;
  transactions: ParsedTransaction[];
  skipped_rows: string[];
}


export interface StagedTransaction extends ParsedTransaction { // extends adds these fields to ParsedTransaction
  category_id: number | null;
  rule_matched: boolean;
}


export interface CategoryRule {
  id: number;
  category_id: number;
  priority: number;

  details_pattern: string | null;
  details_match_type: string | null;      // 'exact', 'contains', 'starts_with'

  description_pattern: string | null;
  description_match_type: string | null;

  type_pattern: string | null;
  type_match_type: string | null;

  amount_operator: string | null;         // '<', '>', '=', '<=', '>='
  amount_value: number | null;            // in cents
}

export interface Category {
  id: number;
  name: string;
}

export interface Account {
  id: number;
  csv_id: string;
  name: string;
}

export interface Transaction {
  id: number;
  account_id: number;
  details: string;
  posting_date: string;
  description: string;
  amount_cents: number;
  transaction_type: string;
  balance_cents: number;
  check_or_slip_num: string | null;
  category_id: number | null;
  notes: string | null;
  imported_at: string;
}
