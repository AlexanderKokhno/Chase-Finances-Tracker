use rusqlite::params;
use serde::Deserialize;
use tauri::Manager;
use std::path::PathBuf;
use std::collections::HashMap;

/// Unassign transactions that belong to a specific category (set category_id to NULL).
/// Call this BEFORE deleting the category to avoid foreign key constraint errors.
fn unassign_category(conn: &rusqlite::Connection, category_id: i64) -> Result<(), String> {
    conn.execute(
        "UPDATE transactions SET category_id = NULL WHERE category_id = ?1",
        params![category_id],
    )
    .map_err(|e| format!("Failed to unassign transactions: {}", e))?;

    Ok(())
}

/// Unassign ALL transactions (set category_id to NULL).
/// Call this BEFORE deleting all categories to avoid foreign key constraint errors.
fn unassign_all_categories(conn: &rusqlite::Connection) -> Result<(), String> {
    conn.execute(
        "UPDATE transactions SET category_id = NULL WHERE category_id IS NOT NULL",
        [],
    )
    .map_err(|e| format!("Failed to unassign transactions: {}", e))?;

    Ok(())
}

/// Matches the StagedTransaction shape sent from the frontend
#[derive(Deserialize)]
pub struct StagedTransaction {
    pub details: String,
    pub posting_date: String,
    pub description: String,
    pub amount_cents: i64,
    pub transaction_type: String,
    pub balance_cents: i64,
    pub check_or_slip_num: String,
    pub category_id: Option<i64>,
}

/// Helper to get the path to our SQLite database file
fn get_db_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("Could not find config dir: {}", e))?;
    Ok(config_dir.join("chase_tracker.db"))
}

#[tauri::command]
pub fn import_transactions(
    app: tauri::AppHandle,
    account_id: String,
    transactions: Vec<StagedTransaction>,
) -> Result<(), String> {
    let db_path = get_db_path(&app)?;

    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Could not open database: {}", e))?;

    // Insert account (or ignore if it already exists)
    conn.execute(
        "INSERT OR IGNORE INTO accounts (csv_id) VALUES (?1)",
        params![account_id],
    )
    .map_err(|e| format!("Failed to insert account: {}", e))?;

    // Get the account's row ID
    let account_row_id: i64 = conn
        .query_row(
            "SELECT id FROM accounts WHERE csv_id = ?1",
            params![account_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to find account: {}", e))?;

    // Insert each transaction (duplicates silently skipped via UNIQUE constraint)
    for tx in &transactions {
        conn.execute(
            "INSERT OR IGNORE INTO transactions
                (account_id, details, posting_date, description, amount_cents,
                 transaction_type, balance_cents, check_or_slip_num, category_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                account_row_id,
                tx.details,
                tx.posting_date,
                tx.description,
                tx.amount_cents,
                tx.transaction_type,
                tx.balance_cents,
                tx.check_or_slip_num,
                tx.category_id,
            ],
        )
        .map_err(|e| format!("Failed to insert transaction: {}", e))?;
    }

    Ok(())
}

/// Category with a frontend-chosen ID
#[derive(Deserialize)]
pub struct NewCategory {
    pub id: i64,
    pub display_name: String,
}

/// Rule to insert — category_id references a category, rule id auto-increments
#[derive(Deserialize)]
pub struct NewCategoryRule {
    pub id: i64,
    pub category_id: i64,
    pub priority: i64,
    pub details_pattern: Option<String>,
    pub details_match_type: Option<String>,
    pub description_pattern: Option<String>,
    pub description_match_type: Option<String>,
    pub type_pattern: Option<String>,
    pub type_match_type: Option<String>,
    pub amount_operator: Option<String>,
    pub amount_value: Option<i64>,
}

#[tauri::command]
pub fn import_category_rules(
    app: tauri::AppHandle,
    categories: Vec<NewCategory>,
    rules: Vec<NewCategoryRule>,
) -> Result<(), String> {
    let db_path = get_db_path(&app)?;

    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Could not open database: {}", e))?;

    // Insert categories — error if the ID is already used by a different category
    for cat in &categories {
        // Check if this ID already exists
        let existing: Option<String> = conn
            .query_row(
                "SELECT display_name FROM categories WHERE id = ?1",
                params![cat.id],
                |row| row.get(0),
            )
            .ok();

        match existing {
            // ID exists with same name — nothing to do
            Some(ref name) if name == &cat.display_name => {}
            // ID exists with different name — error
            Some(ref name) => {
                return Err(format!(
                    "Category ID {} is already used by '{}', cannot create '{}'",
                    cat.id, name, cat.display_name
                ));
            }
            // ID doesn't exist — insert with the frontend-chosen ID
            None => {
                conn.execute(
                    "INSERT INTO categories (id, display_name) VALUES (?1, ?2)",
                    params![cat.id, cat.display_name],
                )
                .map_err(|e| format!("Failed to insert category '{}': {}", cat.display_name, e))?;
            }
        }
    }

    // Insert rules — IDs provided by the frontend (category_id * 1000 + increment)
    for rule in &rules {
        conn.execute(
            "INSERT INTO category_rules
                (id, category_id, priority,
                 details_pattern, details_match_type,
                 description_pattern, description_match_type,
                 type_pattern, type_match_type,
                 amount_operator, amount_value)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                rule.id,
                rule.category_id,
                rule.priority,
                rule.details_pattern,
                rule.details_match_type,
                rule.description_pattern,
                rule.description_match_type,
                rule.type_pattern,
                rule.type_match_type,
                rule.amount_operator,
                rule.amount_value,
            ],
        )
        .map_err(|e| format!("Failed to insert rule (id {}): {}", rule.id, e))?;
    }

    Ok(())
}

/// Structs for the embedded default JSON files
#[derive(Deserialize)]
struct DefaultRule {
    category_name: String,
    priority: i64,
    details_pattern: Option<String>,
    details_match_type: Option<String>,
    description_pattern: Option<String>,
    description_match_type: Option<String>,
    type_pattern: Option<String>,
    type_match_type: Option<String>,
    amount_operator: Option<String>,
    amount_value: Option<i64>,
}

#[tauri::command]
pub fn load_default_rules(app: tauri::AppHandle) -> Result<String, String> {
    let db_path = get_db_path(&app)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Could not open database: {}", e))?;

    // Load the default data embedded at compile time
    let category_names: Vec<String> =
        serde_json::from_str(include_str!("../../src/data/defaults/default_categories.json"))
            .map_err(|e| format!("Failed to parse default categories: {}", e))?;

    let default_rules: Vec<DefaultRule> =
        serde_json::from_str(include_str!("../../src/data/defaults/default_rules.json"))
            .map_err(|e| format!("Failed to parse default rules: {}", e))?;

    // Step 1: Ensure each default category exists, collect name -> id mapping
    let mut category_ids: HashMap<String, i64> = HashMap::new();
    let mut cats_added = 0;

    for name in &category_names {
        // Check if this category name already exists
        let existing_id: Option<i64> = conn
            .query_row(
                "SELECT id FROM categories WHERE display_name = ?1",
                params![name],
                |row| row.get(0),
            )
            .ok();

        let cat_id = match existing_id {
            Some(id) => id,
            None => {
                conn.execute(
                    "INSERT INTO categories (display_name) VALUES (?1)",
                    params![name],
                )
                .map_err(|e| format!("Failed to insert category '{}': {}", name, e))?;
                cats_added += 1;
                conn.last_insert_rowid()
            }
        };

        category_ids.insert(name.clone(), cat_id);
    }

    // Step 2: Insert each default rule, skipping duplicates
    let mut rules_added = 0;

    for rule in &default_rules {
        let cat_id = category_ids
            .get(&rule.category_name)
            .ok_or_else(|| format!("Unknown category name: {}", rule.category_name))?;

        // Check if an identical rule already exists for this category
        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM category_rules
                 WHERE category_id = ?1
                   AND IFNULL(details_pattern, '') = IFNULL(?2, '')
                   AND IFNULL(description_pattern, '') = IFNULL(?3, '')
                   AND IFNULL(type_pattern, '') = IFNULL(?4, '')",
                params![
                    cat_id,
                    rule.details_pattern,
                    rule.description_pattern,
                    rule.type_pattern,
                ],
                |row| row.get::<_, i64>(0),
            )
            .map(|count| count > 0)
            .unwrap_or(false);

        if exists {
            continue;
        }

        conn.execute(
            "INSERT INTO category_rules
                (category_id, priority,
                 details_pattern, details_match_type,
                 description_pattern, description_match_type,
                 type_pattern, type_match_type,
                 amount_operator, amount_value)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                cat_id,
                rule.priority,
                rule.details_pattern,
                rule.details_match_type,
                rule.description_pattern,
                rule.description_match_type,
                rule.type_pattern,
                rule.type_match_type,
                rule.amount_operator,
                rule.amount_value,
            ],
        )
        .map_err(|e| format!("Failed to insert rule: {}", e))?;
        rules_added += 1;
    }

    Ok(format!(
        "Added {} categories, {} rules ({} skipped as duplicates)",
        cats_added,
        rules_added,
        default_rules.len() - rules_added
    ))
}

#[tauri::command]
pub fn delete_category(app: tauri::AppHandle, category_id: i64) -> Result<(), String> {
    let db_path = get_db_path(&app)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Could not open database: {}", e))?;

    // Unassign transactions first so foreign key doesn't block the delete
    unassign_category(&conn, category_id)?;

    // Delete all rules for this category
    conn.execute(
        "DELETE FROM category_rules WHERE category_id = ?1",
        params![category_id],
    )
    .map_err(|e| format!("Failed to delete rules: {}", e))?;

    // Delete the category itself
    conn.execute(
        "DELETE FROM categories WHERE id = ?1",
        params![category_id],
    )
    .map_err(|e| format!("Failed to delete category: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn delete_rule(app: tauri::AppHandle, rule_id: i64) -> Result<(), String> {
    let db_path = get_db_path(&app)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Could not open database: {}", e))?;

    conn.execute(
        "DELETE FROM category_rules WHERE id = ?1",
        params![rule_id],
    )
    .map_err(|e| format!("Failed to delete rule: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn rebuild_database(app: tauri::AppHandle) -> Result<(), String> {
    let db_path = get_db_path(&app)?;

    // Delete the DB file if it exists
    if db_path.exists() {
        std::fs::remove_file(&db_path)
            .map_err(|e| format!("Failed to delete database: {}", e))?;
    }

    // Restart the app — LoadingView will recreate the DB on boot
    app.restart();
}

// ---- Settings-related commands ----

/// Returns the DB file path as a string so the frontend can show/open it
#[tauri::command]
pub fn get_db_path_string(app: tauri::AppHandle) -> Result<String, String> {
    let db_path = get_db_path(&app)?;
    Ok(db_path.to_string_lossy().to_string())
}

/// Delete ALL category rules AND categories
#[tauri::command]
pub fn nuke_all_rules(app: tauri::AppHandle) -> Result<(), String> {
    let db_path = get_db_path(&app)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Could not open database: {}", e))?;

    // Unassign all transactions first so foreign keys don't block the deletes
    unassign_all_categories(&conn)?;

    // Now safe to delete rules and categories
    conn.execute("DELETE FROM category_rules", [])
        .map_err(|e| format!("Failed to delete rules: {}", e))?;

    conn.execute("DELETE FROM categories", [])
        .map_err(|e| format!("Failed to delete categories: {}", e))?;

    Ok(())
}

/// Delete an account and all its transactions
#[tauri::command]
pub fn delete_account(app: tauri::AppHandle, account_id: i64) -> Result<(), String> {
    let db_path = get_db_path(&app)?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Could not open database: {}", e))?;

    // Delete all transactions for this account first
    conn.execute(
        "DELETE FROM transactions WHERE account_id = ?1",
        params![account_id],
    )
    .map_err(|e| format!("Failed to delete transactions: {}", e))?;

    // Delete the account itself
    conn.execute(
        "DELETE FROM accounts WHERE id = ?1",
        params![account_id],
    )
    .map_err(|e| format!("Failed to delete account: {}", e))?;

    Ok(())
}

// ---- Database export / import ----

/// Export the database to a user-chosen destination.
/// Uses VACUUM INTO so the export includes all WAL data in a single file.
#[tauri::command]
pub fn export_database(app: tauri::AppHandle, dest_path: String) -> Result<(), String> {
    let db_path = get_db_path(&app)?;
    if !db_path.exists() {
        return Err("Database file not found".to_string());
    }

    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Could not open database: {}", e))?;

    // VACUUM INTO creates a complete standalone copy that includes any
    // unflushed WAL data, so the exported file is always up to date.
    conn.execute_batch(&format!("VACUUM INTO '{}';", dest_path.replace('\'', "''")))
        .map_err(|e| format!("Failed to export database: {}", e))?;

    Ok(())
}

/// Replace the current database with a user-chosen file, then restart
#[tauri::command]
pub fn import_database(app: tauri::AppHandle, source_path: String) -> Result<(), String> {
    let db_path = get_db_path(&app)?;
    let source = std::path::Path::new(&source_path);

    if !source.exists() {
        return Err("Selected file not found".to_string());
    }

    // Quick sanity check: try opening the file as a SQLite database
    let test_conn = rusqlite::Connection::open(source)
        .map_err(|_| "The selected file is not a valid SQLite database".to_string())?;
    // Check that it has the expected tables
    let has_accounts: bool = test_conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='accounts'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map(|count| count > 0)
        .unwrap_or(false);
    // Close the test connection before copying
    drop(test_conn);

    if !has_accounts {
        return Err("The selected file doesn't look like a Chase Tracker database".to_string());
    }

    // Copy the file over the current database
    std::fs::copy(source, &db_path)
        .map_err(|e| format!("Failed to import database: {}", e))?;

    // Remove leftover WAL/SHM files from the old database session.
    // If these stick around, SQLite will replay stale WAL data on top
    // of our freshly imported file when the app restarts.
    let wal_path = db_path.with_extension("db-wal");
    let shm_path = db_path.with_extension("db-shm");
    let _ = std::fs::remove_file(wal_path);
    let _ = std::fs::remove_file(shm_path);

    // Restart the app so it picks up the new database
    app.restart();
}

// ---- Custom loading assets ----

/// Helper to get the user_custom directory path
fn get_user_custom_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("Could not find config dir: {}", e))?;
    Ok(config_dir.join("user_custom"))
}

/// Read custom tips from user_custom/tips.json
#[tauri::command]
pub fn get_custom_tips(app: tauri::AppHandle) -> Result<Vec<String>, String> {
    let tips_path = get_user_custom_dir(&app)?.join("tips.json");
    if !tips_path.exists() {
        return Ok(vec![]);
    }
    let content = std::fs::read_to_string(&tips_path)
        .map_err(|e| format!("Failed to read tips: {}", e))?;
    let tips: Vec<String> = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse tips: {}", e))?;
    Ok(tips)
}

/// Add a custom tip to tips.json
#[tauri::command]
pub fn add_custom_tip(app: tauri::AppHandle, text: String) -> Result<(), String> {
    let custom_dir = get_user_custom_dir(&app)?;
    std::fs::create_dir_all(&custom_dir)
        .map_err(|e| format!("Failed to create directory: {}", e))?;

    let tips_path = custom_dir.join("tips.json");
    let mut tips = if tips_path.exists() {
        let content = std::fs::read_to_string(&tips_path)
            .map_err(|e| format!("Failed to read tips: {}", e))?;
        serde_json::from_str::<Vec<String>>(&content)
            .map_err(|e| format!("Failed to parse tips: {}", e))?
    } else {
        vec![]
    };

    tips.push(text);
    let json = serde_json::to_string_pretty(&tips)
        .map_err(|e| format!("Failed to serialize tips: {}", e))?;
    std::fs::write(&tips_path, json)
        .map_err(|e| format!("Failed to write tips: {}", e))?;
    Ok(())
}

/// Remove a custom tip by index
#[tauri::command]
pub fn remove_custom_tip(app: tauri::AppHandle, index: usize) -> Result<(), String> {
    let tips_path = get_user_custom_dir(&app)?.join("tips.json");
    if !tips_path.exists() {
        return Err("No custom tips file found".to_string());
    }

    let content = std::fs::read_to_string(&tips_path)
        .map_err(|e| format!("Failed to read tips: {}", e))?;
    let mut tips: Vec<String> = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse tips: {}", e))?;

    if index >= tips.len() {
        return Err("Tip index out of range".to_string());
    }

    tips.remove(index);
    let json = serde_json::to_string_pretty(&tips)
        .map_err(|e| format!("Failed to serialize tips: {}", e))?;
    std::fs::write(&tips_path, json)
        .map_err(|e| format!("Failed to write tips: {}", e))?;
    Ok(())
}

/// List custom loading images (filenames only)
#[tauri::command]
pub fn list_loading_images(app: tauri::AppHandle) -> Result<Vec<String>, String> {
    let images_dir = get_user_custom_dir(&app)?.join("images");
    if !images_dir.exists() {
        return Ok(vec![]);
    }

    let mut names = vec![];
    let entries = std::fs::read_dir(&images_dir)
        .map_err(|e| format!("Failed to read images directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        if let Some(name) = entry.file_name().to_str() {
            names.push(name.to_string());
        }
    }
    Ok(names)
}

/// Copy an image file into user_custom/images/
#[tauri::command]
pub fn add_loading_image(app: tauri::AppHandle, source_path: String) -> Result<String, String> {
    let images_dir = get_user_custom_dir(&app)?.join("images");
    std::fs::create_dir_all(&images_dir)
        .map_err(|e| format!("Failed to create images directory: {}", e))?;

    let source = std::path::Path::new(&source_path);
    let filename = source
        .file_name()
        .ok_or("Invalid file path")?
        .to_string_lossy()
        .to_string();

    let dest = images_dir.join(&filename);
    std::fs::copy(source, &dest)
        .map_err(|e| format!("Failed to copy image: {}", e))?;

    Ok(filename)
}

/// Remove a custom loading image by filename
#[tauri::command]
pub fn remove_loading_image(app: tauri::AppHandle, filename: String) -> Result<(), String> {
    let images_dir = get_user_custom_dir(&app)?.join("images");
    let file_path = images_dir.join(&filename);

    if !file_path.exists() {
        return Err(format!("Image '{}' not found", filename));
    }

    std::fs::remove_file(&file_path)
        .map_err(|e| format!("Failed to remove image: {}", e))?;
    Ok(())
}
