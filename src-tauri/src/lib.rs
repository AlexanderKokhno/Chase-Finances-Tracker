mod csv_parser;
mod db_utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_sql::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            csv_parser::parse_chase_csv,
            db_utils::rebuild_database,
            db_utils::import_transactions,
            db_utils::import_category_rules,
            db_utils::load_default_rules,
            db_utils::delete_category,
            db_utils::delete_rule,
            db_utils::get_db_path_string,
            db_utils::nuke_all_rules,
            db_utils::delete_account,
            db_utils::export_database,
            db_utils::import_database,
            db_utils::get_custom_tips,
            db_utils::add_custom_tip,
            db_utils::remove_custom_tip,
            db_utils::list_loading_images,
            db_utils::add_loading_image,
            db_utils::remove_loading_image
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
