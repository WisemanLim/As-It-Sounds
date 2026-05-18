mod commands;
pub mod core;

use std::sync::Arc;

use tauri::Manager;

use crate::commands::AppState;
use crate::core::asr::AsrEngine;
use crate::core::db::Db;
use crate::core::pipeline::Pipeline;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let handle = app.handle();
            let data_dir = handle
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::env::temp_dir().join("as-it-sounds"));
            std::fs::create_dir_all(&data_dir).ok();
            let db_path = data_dir.join("as-it-sounds.sqlite");
            let db = Db::open(&db_path).expect("open db");
            let models_dir = data_dir.join("models");
            let asr = Arc::new(AsrEngine::new(models_dir).expect("init asr"));
            let pipeline =
                Arc::new(Pipeline::new(db.clone(), asr.clone()).expect("init pipeline"));
            app.manage(AppState { db, pipeline });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::project_list,
            commands::project_create,
            commands::project_rename,
            commands::project_delete,
            commands::file_list,
            commands::file_add_many,
            commands::file_remove,
            commands::file_reset,
            commands::segments_list,
            commands::settings_get,
            commands::settings_set,
            commands::pipeline_run,
            commands::translit_preview,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
