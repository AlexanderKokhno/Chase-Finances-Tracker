# Chase Finances Tracker

A desktop app for importing, categorizing, and visualizing your Chase bank statement transactions.

[![Download Latest Release](https://img.shields.io/github/v/release/AlexanderKokhno/Chase-Finances-Tracker?label=Download&style=for-the-badge)](https://github.com/AlexanderKokhno/Chase-Finances-Tracker/releases/latest)

## Download

Go to the [Releases](https://github.com/AlexanderKokhno/Chase-Finances-Tracker/releases/latest) page and download the installer for your platform:

| Platform | File |
| -------- | ---- |
| Windows  | `.msi` or `.exe` |
| Linux (Debian/Ubuntu/Mint) | `.deb` |
| Linux (Other) | `.AppImage` |

## Features

- **CSV Import** — Import Chase bank statement CSVs with a staged review before committing
- **Transaction Management** — View, search, filter, and sort transactions across multiple accounts
- **Rule-Based Categorization** — Auto-categorize transactions using priority-ordered rules
- **Category Management** — Create and manage custom spending/income categories
- **Dashboard and Charts** — Balance trends, cash flow charts, and candlestick views
- **Multi-Account Support** — Track multiple Chase accounts with custom nicknames
- **Dark/Light Theme** — Toggle between themes in settings

## How It Works

1. **Import** — Select a Chase CSV file, the app parses and stages transactions for review
2. **Categorize** — Rules auto-categorize by description, type, amount, etc. Override manually if needed
3. **Analyze** — View trends and breakdowns on the dashboard

## Built With

- [Tauri 2](https://v2.tauri.app/) (Rust backend)
- [Vue 3](https://vuejs.org/) (Frontend)
- SQLite (Local database)
