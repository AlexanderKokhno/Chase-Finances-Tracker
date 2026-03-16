# Chase Income Tracker

A desktop app for importing, categorizing, and visualizing Chase bank statement transactions. Built with Tauri, Vue 3, and Rust.

## Features

- **CSV Import** — Import Chase bank statement CSVs with a staged review workflow before committing to the database
- **Transaction Management** — View, search, filter, and sort transactions across multiple accounts
- **Rule-Based Categorization** — Priority-ordered rules auto-categorize transactions by description, type, amount, etc.
- **Category Management** — Create and manage custom spending/income categories
- **Dashboard & Charts** — Balance trends, cash flow bar charts, and candlestick views powered by ECharts
- **Multi-Account Support** — Track multiple Chase accounts with custom nicknames
- **Settings** — Dark/light theme toggle, window size persistence, database management

## Tech Stack

| Layer    | Technology                                      |
| -------- | ----------------------------------------------- |
| Frontend | Vue 3, TypeScript, Vue Router, Vite             |
| Backend  | Rust, Tauri 2, rusqlite                         |
| Database | SQLite (via tauri-plugin-sql + rusqlite)         |
| Charts   | ECharts + Vue ECharts                           |
| Icons    | Lucide Vue Next                                 |
| Plugins  | tauri-plugin-dialog, tauri-plugin-sql, tauri-plugin-opener |

## Prerequisites

- [Node.js](https://nodejs.org/) (LTS recommended)
- [Rust](https://www.rust-lang.org/tools/install)
- Tauri v2 system dependencies — see [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)

## Getting Started

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

## Project Structure

```
src/                        # Vue 3 frontend
  views/                    # Page components (Dashboard, Import, Transactions, etc.)
  services/                 # Business logic (database, CSV import, category matching)
  composables/              # Vue composables
  types.ts                  # TypeScript interfaces
  schema.sql                # SQLite schema
  router.ts                 # Vue Router config

src-tauri/                  # Rust backend
  src/
    csv_parser.rs           # Chase CSV parsing
    db_utils.rs             # Database operations
    lib.rs                  # Tauri builder + plugin setup
    main.rs                 # Entry point
```

## How It Works

1. **Import** — Select a Chase CSV file, the app parses it and stages transactions for review
2. **Categorize** — Rules are evaluated by priority (highest first); all conditions on a rule must match (AND logic). First matching rule wins
3. **Review** — Override categories manually before confirming the import
4. **Analyze** — View trends and breakdowns on the dashboard

All monetary amounts are stored as integer cents to avoid floating-point precision issues. Duplicate transactions are automatically skipped via unique constraints.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
