use crate::copy::{copy_skills, ConflictPolicy, CopyResult};
use crate::folder::{list_folder_files, read_folder_file};
use crate::scan::scan_skills;
use crate::skill::default_ignore_patterns;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, State};

const STORE_PATH: &str = "settings.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderFileInfo {
    pub relative_path: String,
    pub absolute_path: String,
    pub size_bytes: u64,
    pub is_binary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileContentResult {
    pub content: String,
    pub is_binary: bool,
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub scan_root: Option<String>,
    pub recent_destinations: Vec<String>,
    pub ignore_patterns: Vec<String>,
}

pub struct AppState {
    pub scan_root: Mutex<Option<PathBuf>>,
    pub ignore_patterns: Mutex<Vec<String>>,
    pub scan_cancel: Mutex<Option<Arc<AtomicBool>>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            scan_root: Mutex::new(None),
            ignore_patterns: Mutex::new(default_ignore_patterns()),
            scan_cancel: Mutex::new(None),
        }
    }
}

fn load_settings(app: &AppHandle) -> Result<AppSettings, String> {
    use tauri_plugin_store::StoreExt;

    let store = app
        .store(STORE_PATH)
        .map_err(|e| format!("Failed to open store: {e}"))?;

    Ok(AppSettings {
        scan_root: store
            .get("scanRoot")
            .and_then(|v| v.as_str().map(String::from)),
        recent_destinations: store
            .get("recentDestinations")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default(),
        ignore_patterns: store
            .get("ignorePatterns")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_else(default_ignore_patterns),
    })
}

fn save_settings(app: &AppHandle, settings: &AppSettings) -> Result<(), String> {
    use tauri_plugin_store::StoreExt;

    let store = app
        .store(STORE_PATH)
        .map_err(|e| format!("Failed to open store: {e}"))?;

    store.set("scanRoot", settings.scan_root.clone());
    store.set(
        "recentDestinations",
        serde_json::to_value(&settings.recent_destinations).unwrap(),
    );
    store.set(
        "ignorePatterns",
        serde_json::to_value(&settings.ignore_patterns).unwrap(),
    );
    store
        .save()
        .map_err(|e| format!("Failed to save settings: {e}"))
}

pub fn init_settings(app: &AppHandle, state: &AppState) {
    if let Ok(settings) = load_settings(app) {
        if let Some(ref root) = settings.scan_root {
            if let Ok(mut scan_root) = state.scan_root.lock() {
                *scan_root = Some(PathBuf::from(root));
            }
        }
        if !settings.ignore_patterns.is_empty() {
            if let Ok(mut ignores) = state.ignore_patterns.lock() {
                *ignores = settings.ignore_patterns;
            }
        }
    }
}

#[tauri::command]
pub fn get_scan_root(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let root = state.scan_root.lock().map_err(|e| e.to_string())?;
    Ok(root.as_ref().map(|p| p.to_string_lossy().to_string()))
}

#[tauri::command]
pub fn set_scan_root(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> Result<(), String> {
    let path_buf = PathBuf::from(&path);
    if !path_buf.is_dir() {
        return Err(format!("Not a directory: {path}"));
    }

    {
        let mut root = state.scan_root.lock().map_err(|e| e.to_string())?;
        *root = Some(path_buf);
    }

    let mut settings = load_settings(&app)?;
    settings.scan_root = Some(path);
    save_settings(&app, &settings)
}

#[tauri::command]
pub fn get_settings(app: AppHandle, state: State<'_, AppState>) -> Result<AppSettings, String> {
    let settings = load_settings(&app)?;

    if let Some(ref root) = settings.scan_root {
        let mut scan_root = state.scan_root.lock().map_err(|e| e.to_string())?;
        *scan_root = Some(PathBuf::from(root));
    }

    if !settings.ignore_patterns.is_empty() {
        let mut ignores = state.ignore_patterns.lock().map_err(|e| e.to_string())?;
        *ignores = settings.ignore_patterns.clone();
    }

    Ok(settings)
}

#[tauri::command]
pub fn add_recent_destination(app: AppHandle, path: String) -> Result<(), String> {
    let mut settings = load_settings(&app)?;
    settings.recent_destinations.retain(|p| p != &path);
    settings.recent_destinations.insert(0, path);
    settings.recent_destinations.truncate(10);
    save_settings(&app, &settings)
}

#[tauri::command]
pub async fn start_scan(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let root = {
        let root_guard = state.scan_root.lock().map_err(|e| e.to_string())?;
        root_guard
            .clone()
            .ok_or_else(|| "No scan root set".to_string())?
    };

    let ignore_patterns = {
        let ignores = state.ignore_patterns.lock().map_err(|e| e.to_string())?;
        ignores.clone()
    };

    let cancel_flag = Arc::new(AtomicBool::new(false));
    {
        let mut cancel = state.scan_cancel.lock().map_err(|e| e.to_string())?;
        *cancel = Some(cancel_flag.clone());
    }

    let app_handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let result = scan_skills(&root, &ignore_patterns, cancel_flag);
        let _ = app_handle.emit("scan-complete", &result);
    });

    Ok(())
}

#[tauri::command]
pub fn cancel_scan(state: State<'_, AppState>) -> Result<(), String> {
    let cancel = state.scan_cancel.lock().map_err(|e| e.to_string())?;
    if let Some(flag) = cancel.as_ref() {
        flag.store(true, Ordering::Relaxed);
    }
    Ok(())
}

#[tauri::command]
pub fn copy_skills_command(
    sources: Vec<String>,
    dest: String,
    on_conflict: ConflictPolicy,
    copy_to_all_skill_folders: Option<bool>,
) -> Result<Vec<CopyResult>, String> {
    let source_paths: Vec<PathBuf> = sources.into_iter().map(PathBuf::from).collect();
    let dest_path = PathBuf::from(&dest);
    Ok(copy_skills(
        &source_paths,
        &dest_path,
        on_conflict,
        copy_to_all_skill_folders.unwrap_or(false),
    ))
}

#[tauri::command]
pub fn find_skill_containers_command(root: String) -> Result<Vec<String>, String> {
    use crate::copy::find_skill_containers;

    Ok(find_skill_containers(&PathBuf::from(&root))
        .into_iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect())
}

#[tauri::command]
pub fn list_skill_files_command(skill_path: String) -> Result<Vec<FolderFileInfo>, String> {
    let files = list_folder_files(&PathBuf::from(&skill_path)).map_err(|e| e.to_string())?;
    Ok(files
        .into_iter()
        .map(|f| FolderFileInfo {
            relative_path: f.relative_path,
            absolute_path: f.absolute_path,
            size_bytes: f.size_bytes,
            is_binary: f.is_binary,
        })
        .collect())
}

#[tauri::command]
pub fn read_skill_file_command(
    path: String,
    max_bytes: Option<usize>,
) -> Result<FileContentResult, String> {
    let (content, is_binary, truncated) =
        read_folder_file(&PathBuf::from(&path), max_bytes.unwrap_or(512 * 1024))
            .map_err(|e| e.to_string())?;
    Ok(FileContentResult {
        content,
        is_binary,
        truncated,
    })
}
