use serde::Serialize;
use std::fs;
use std::path::Path;

const EXPECTED_COLUMNS: usize = 7;

#[derive(Serialize)]
pub struct ParsedTransaction {
    pub details: String,
    pub posting_date: String,
    pub description: String,
    pub amount_cents: i64,
    pub transaction_type: String,
    pub balance_cents: i64,
    pub check_or_slip_num: String,
}

#[derive(Serialize)]
pub struct CsvParseData {
    pub account_id: String,
    pub transactions: Vec<ParsedTransaction>,
    pub skipped_rows: Vec<String>,
}


fn dollars_to_cents(s: &str) -> Result<i64, String> {
    let cleaned = s.trim().replace(",", "");
    let dollars: f64 = cleaned.parse()
        .map_err(|_| format!("Could not parse amount: '{}'", s))?;
    Ok((dollars * 100.0).round() as i64)
}

fn extract_account_id(file_path: &Path) -> Result<String, String> {  // -> "Chase####"
    let file_name = file_path.file_name()
        .and_then(|n| n.to_str())
        .ok_or("Could not read filename")?;

    match file_name.split('_').next() {
        Some(id) => Ok(id.to_string()),
        None => Err("Could not extract account ID from filename".to_string()),
    }
}

#[tauri::command]
pub fn parse_chase_csv(file_path: String) -> Result<CsvParseData, String> { // If extracting a row fails, the entire thing kis
    let path = Path::new(&file_path);

    if !path.exists() {
        return Err(format!("File not found: {}", file_path));
    }

    let account_id = extract_account_id(path)?;


    let contents = fs::read_to_string(path)
        .map_err(|e| format!("Could not read file: {}", e))?;


    // Validate header row has the expected number of columns
    if let Some(header) = contents.lines().next() {
        let column_count = header.split(',').count();
        if column_count != EXPECTED_COLUMNS {
            return Err(format!(
                "CSV format mismatch: expected {} columns but header has {}. Chase may have changed their export format.",
                EXPECTED_COLUMNS, column_count
            ));
        }
    } else {
        return Err("CSV file is empty".to_string());
    }

    let mut transactions = Vec::new();
    let mut skipped_rows: Vec<String> = Vec::new();

    for (row_num, line) in contents.lines().skip(1).enumerate() {     // Skip the header row
        let row_num = row_num + 2; // +2 because we skipped the header (row 1) and rows are 1-indexed
        if line.trim().is_empty() {
            continue;
        }

        let fields: Vec<&str> = line.splitn(7, ',').collect();
        if fields.len() < 7 {
            let msg = format!("Row {}: expected 7 fields, got {}", row_num, fields.len());
            eprintln!("WARNING: {}", msg);
            skipped_rows.push(msg);
            continue;
        }

        let amount_cents = match dollars_to_cents(fields[3]) {
            Ok(val) => val,
            Err(e) => {
                let msg = format!("Row {}: {} ({})", row_num, e, fields[2].trim());
                eprintln!("WARNING: Skipping - {}", msg);
                skipped_rows.push(msg);
                continue;
            }
        };

        let balance_cents = match dollars_to_cents(fields[5]) {
            Ok(val) => val,
            Err(e) => {
                let msg = format!("Row {}: {} ({})", row_num, e, fields[2].trim());
                eprintln!("WARNING: Skipping - {}", msg);
                skipped_rows.push(msg);
                continue;
            }
        };

        let transaction = ParsedTransaction {
            details: fields[0].trim().to_string(),
            posting_date: fields[1].trim().to_string(),
            description: fields[2].trim().to_string(),
            amount_cents,
            transaction_type: fields[4].trim().to_string(),
            balance_cents,
            check_or_slip_num: fields[6].trim().to_string(),
        };

        transactions.push(transaction);
    }

    Ok(CsvParseData {
        account_id,
        transactions,
        skipped_rows,
    })
}