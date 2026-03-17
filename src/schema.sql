-- Chase Income Tracker Database Schema
-- Minimal, data-focused design

PRAGMA foreign_keys = ON;

-- Accounts — one row per Chase account (last 4 digits from CSV filename)
-- e.g. filename "Chase3237_Activity_20260129.CSV" → csv_id = "Chase3237"
CREATE TABLE IF NOT EXISTS accounts (
    id INTEGER PRIMARY KEY,
    csv_id TEXT NOT NULL UNIQUE,       -- "Chase3237" — unique per account
    display_name TEXT                   -- User-assigned friendly name (e.g. 'Main Checking')
);

-- Categories — master list of spending/income categories
-- e.g. "Income", "Subscriptions", "Groceries", "Transfers"
CREATE TABLE IF NOT EXISTS categories (
    id INTEGER PRIMARY KEY,
    display_name TEXT NOT NULL UNIQUE           -- Category display name
);

-- Category rules — auto-assign categories during CSV import
-- e.g. pattern "ADP" matches "ADP PAYROLL" → assigns category_id for "Income"
CREATE TABLE IF NOT EXISTS category_rules (
    id INTEGER PRIMARY KEY,
    category_id INTEGER NOT NULL REFERENCES categories(id),
    priority INTEGER NOT NULL DEFAULT 0,

    -- Each field is optional. NULL = don't check this field.
    details_pattern TEXT,            -- e.g. 'CREDIT'
    details_match_type TEXT,         -- 'exact', 'contains', 'starts_with'

    description_pattern TEXT,        -- e.g. 'SPOTIFY'
    description_match_type TEXT,

    type_pattern TEXT,               -- e.g. 'ACCT_XFER'
    type_match_type TEXT,

    amount_operator TEXT,            -- '<', '>', '=', '<=', '>='
    amount_value INTEGER             -- in cents, like your transactions
);

-- Transactions imported from Chase CSV
-- Amounts stored as INTEGER cents to avoid floating point errors
CREATE TABLE IF NOT EXISTS transactions (
    id INTEGER PRIMARY KEY,
    account_id INTEGER NOT NULL REFERENCES accounts(id),

    -- Original CSV columns (mapped directly)
    details TEXT NOT NULL,              -- CREDIT, DEBIT, DSLIP
    posting_date TEXT NOT NULL,         -- MM/DD/YYYY from CSV
    description TEXT NOT NULL,          -- Merchant/transaction description
    amount_cents INTEGER NOT NULL,      -- Stored as cents (multiply by 100)
    transaction_type TEXT NOT NULL,     -- ACH_CREDIT, ACCT_XFER, ATM, etc.
    balance_cents INTEGER NOT NULL,     -- Running balance as cents
    check_or_slip_num TEXT,                 -- Usually empty

    -- App-added fields
    category_id INTEGER REFERENCES categories(id),  -- NULL = uncategorized
    notes TEXT,
    imported_at TEXT DEFAULT CURRENT_TIMESTAMP,

    -- Deduplication: same transaction on same account can't be imported twice
    UNIQUE(account_id, posting_date, description, amount_cents, balance_cents)
);

CREATE INDEX IF NOT EXISTS idx_transactions_account ON transactions(account_id);
CREATE INDEX IF NOT EXISTS idx_transactions_date ON transactions(posting_date);
CREATE INDEX IF NOT EXISTS idx_transactions_category ON transactions(category_id);

-- App settings — simple key-value store for user preferences
CREATE TABLE IF NOT EXISTS app_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
