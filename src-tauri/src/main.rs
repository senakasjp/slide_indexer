#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod error;
mod models;
mod scanner;
mod state;

use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::Arc,
};

use tauri::{async_runtime, AppHandle, Manager, State};

use crate::{
    models::{AppState, ScanSummary, SearchResponse},
    state::StateManager,
};

type CommandResult<T> = std::result::Result<T, String>;

#[tauri::command]
fn fetch_state(manager: State<Arc<StateManager>>) -> CommandResult<AppState> {
    Ok(manager.get_state())
}

#[tauri::command]
async fn update_directories(
    manager: State<'_, Arc<StateManager>>,
    directories: Vec<String>,
) -> CommandResult<ScanSummary> {
    let manager = Arc::clone(manager.inner());
    async_runtime::spawn_blocking(move || manager.update_directories(directories))
        .await
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn rescan(manager: State<'_, Arc<StateManager>>) -> CommandResult<ScanSummary> {
    let manager = Arc::clone(manager.inner());
    async_runtime::spawn_blocking(move || manager.rescan())
        .await
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn rescan_directory(
    manager: State<'_, Arc<StateManager>>,
    directory: String,
) -> CommandResult<ScanSummary> {
    let manager = Arc::clone(manager.inner());
    async_runtime::spawn_blocking(move || manager.rescan_directory(directory))
        .await
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn search_index(
    manager: State<Arc<StateManager>>,
    query: Option<String>,
) -> CommandResult<SearchResponse> {
    let query = query.unwrap_or_default();
    Ok(manager.search(&query))
}

#[tauri::command]
fn open_slide_deck(
    _app: AppHandle,
    manager: State<Arc<StateManager>>,
    id: String,
) -> CommandResult<()> {
    let Some(item) = manager.find_item(&id) else {
        return Err("Slide deck not found".to_string());
    };

    let path = PathBuf::from(item.path);
    if !path.exists() {
        return Err("Slide deck path no longer exists".to_string());
    }

    launch_file(path.as_path()).map_err(|error| error.to_string())
}

#[tauri::command]
fn clear_cache(manager: State<Arc<StateManager>>) -> CommandResult<()> {
    manager.clear_cache().map_err(|error| error.to_string())
}

fn launch_file(path: &Path) -> Result<(), std::io::Error> {
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map(|_| ())
    }

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;

        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let path_str = path.to_string_lossy().into_owned();
        let quoted_path = format!("\"{}\"", path_str);
        let mut command = Command::new("cmd");
        command.args(["/C", "start", "", &quoted_path]);
        command.creation_flags(CREATE_NO_WINDOW);
        command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map(|_| ())
    }

    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    {
        Command::new("xdg-open")
            .arg(path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map(|_| ())
    }
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let manager = Arc::new(
                StateManager::new(&app.handle())
                    .map_err(|error| -> Box<dyn std::error::Error> { Box::new(error) })?,
            );
            app.manage(manager);
            
            #[cfg(debug_assertions)]
            {
                let window = app.get_window("main").unwrap();
                window.open_devtools();
            }
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            fetch_state,
            update_directories,
            rescan,
            rescan_directory,
            search_index,
            open_slide_deck,
            clear_cache
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
