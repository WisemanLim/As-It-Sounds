use std::sync::Arc;

use tauri::{AppHandle, Manager, State};

use crate::core::db::Db;
use crate::core::error::AppResult;
use crate::core::models::{
    FileStatus, MediaFile, MediaKind, NewFileInput, Project, Segment, Settings,
};
use crate::core::pipeline::Pipeline;
use crate::core::settings as settings_store;
use crate::core::translit;

pub struct AppState {
    pub db: Db,
    pub pipeline: Arc<Pipeline>,
}

#[tauri::command]
pub async fn project_list(state: State<'_, AppState>) -> AppResult<Vec<Project>> {
    state.db.list_projects()
}

#[tauri::command]
pub async fn project_create(
    state: State<'_, AppState>,
    name: Option<String>,
) -> AppResult<Project> {
    state.db.create_project(name)
}

#[tauri::command]
pub async fn project_rename(
    state: State<'_, AppState>,
    id: String,
    name: String,
) -> AppResult<()> {
    state.db.rename_project(&id, &name)
}

#[tauri::command]
pub async fn project_delete(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<()> {
    state.db.delete_project(&id)
}

#[tauri::command]
pub async fn file_list(
    state: State<'_, AppState>,
    project_id: String,
) -> AppResult<Vec<MediaFile>> {
    state.db.list_files(&project_id)
}

#[tauri::command]
pub async fn file_add_many(
    state: State<'_, AppState>,
    project_id: String,
    inputs: Vec<NewFileInput>,
) -> AppResult<Vec<MediaFile>> {
    let prepared: Vec<_> = inputs
        .into_iter()
        .map(|i| {
            let kind = i
                .kind
                .as_deref()
                .map(|k| match k {
                    "video" => MediaKind::Video,
                    _ => MediaKind::Audio,
                })
                .unwrap_or_else(|| MediaKind::from_name(&i.name));
            (i.name, i.path, kind, i.size_bytes)
        })
        .collect();
    state.db.add_files(&project_id, &prepared)
}

#[tauri::command]
pub async fn file_remove(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<()> {
    state.db.remove_file(&id)
}

#[tauri::command]
pub async fn file_reset(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<()> {
    state.db.clear_segments(&id)?;
    state.db.set_file_status(&id, FileStatus::Pending, None)?;
    Ok(())
}

#[tauri::command]
pub async fn segments_list(
    state: State<'_, AppState>,
    file_id: String,
) -> AppResult<Vec<Segment>> {
    state.db.list_segments(&file_id)
}

#[tauri::command]
pub async fn settings_get(state: State<'_, AppState>) -> AppResult<Settings> {
    settings_store::load(&state.db)
}

#[tauri::command]
pub async fn settings_set(
    state: State<'_, AppState>,
    settings: Settings,
) -> AppResult<Settings> {
    settings_store::save(&state.db, &settings)?;
    Ok(settings)
}

#[tauri::command]
pub async fn pipeline_run(
    app: AppHandle,
    state: State<'_, AppState>,
    file_ids: Vec<String>,
) -> AppResult<()> {
    let settings = settings_store::load(&state.db)?;
    let pipeline = state.pipeline.clone();
    tauri::async_runtime::spawn(async move {
        let _ = pipeline.run(app, file_ids, settings).await;
    });
    Ok(())
}

#[tauri::command]
pub fn translit_preview(text: String) -> String {
    translit::transliterate(&text)
}

#[tauri::command]
pub fn convert_file_src_helper(app: AppHandle, path: String) -> AppResult<String> {
    // Returns a tauri asset URL we can serve from the frontend.
    let _ = app; // reserved for future use
    Ok(path)
}
