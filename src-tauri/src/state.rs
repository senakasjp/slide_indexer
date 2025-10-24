use std::{
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};

use tauri::{AppHandle, Manager};

use crate::{
    error::{AppError, Result},
    models::{AppState, ScanProgressPayload, ScanSummary, SearchResponse, SlideIndexItem},
    scanner::{
        current_timestamp, matches_query, ocr_status_message, scan_directories, ScanOutcome,
        SearchPattern,
    },
};

pub struct StateManager {
    state: Mutex<AppState>,
    storage_path: PathBuf,
    app_handle: AppHandle,
}

impl StateManager {
    pub fn new(handle: &AppHandle) -> Result<Self> {
        let resolver = handle.path_resolver();
        let base_dir = resolver
            .app_data_dir()
            .or_else(|| resolver.app_config_dir())
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

        let data_dir = base_dir.join("slides-indexer");
        fs::create_dir_all(&data_dir)?;

        let storage_path = data_dir.join("index.json");
        let state = if storage_path.exists() {
            load_state(&storage_path)?
        } else {
            let initial = AppState::default();
            persist_state(&storage_path, &initial)?;
            initial
        };

        Ok(Self {
            state: Mutex::new(state),
            storage_path,
            app_handle: handle.clone(),
        })
    }

    pub fn get_state(&self) -> AppState {
        let mut state = self.state.lock().expect("state poisoned").clone();
        println!("get_state returning directories: {:?}", state.directories);
        if let Some(message) = ocr_status_message() {
            if !state.warnings.iter().any(|existing| existing == &message) {
                state.warnings.push(message);
            }
        }
        state
    }

    pub fn rescan(&self) -> Result<ScanSummary> {
        let (directories, existing_snapshot) = {
            let state = self.state.lock().expect("state poisoned");
            (state.directories.clone(), state.items.clone())
        };
        if directories.is_empty() {
            let mut state = self.state.lock().expect("state poisoned");
            state.items.clear();
            state.last_indexed_at = Some(current_timestamp());
            let mut summary = ScanSummary {
                indexed: 0,
                scanned: None,
                cached: None,
                errors: Vec::new(),
                last_indexed_at: state.last_indexed_at,
            };
            if let Some(message) = ocr_status_message() {
                summary.errors.push(message);
            }
            state.warnings = summary.errors.clone();
            let persist_result = persist_state(&self.storage_path, &state);
            self.emit_scan_progress(None, None, None);
            persist_result?;
            return Ok(summary);
        }

        // Create callback that saves state after each indexed file
        let storage_path = self.storage_path.clone();
        let state_mutex = &self.state;
        
        let mut progress_cb = |path: &str, status: &str, debug: Option<&str>| self.emit_scan_progress(Some(path), Some(status), debug);
        
        let mut on_item_indexed = |item: crate::models::SlideIndexItem| {
            let mut state = state_mutex.lock().expect("state poisoned");
            // Add or update the item
            if let Some(pos) = state.items.iter().position(|i| i.path == item.path) {
                state.items[pos] = item;
            } else {
                state.items.push(item);
            }
            state.last_indexed_at = Some(current_timestamp());
            // Save immediately after each file
            if let Err(e) = persist_state(&storage_path, &state) {
                println!("⚠️  Failed to save cache after indexing file: {}", e);
            } else {
                println!("💾 Cache saved (items: {})", state.items.len());
            }
        };
        
        let outcome = scan_directories(&directories, &existing_snapshot, &mut progress_cb, &mut on_item_indexed);
        let ScanOutcome { items, errors, scanned_count, cached_count } = match outcome {
            Ok(result) => result,
            Err(error) => {
                self.emit_scan_progress(None, None, None);
                return Err(error);
            }
        };

        let mut state = self.state.lock().expect("state poisoned");
        state.items = items;
        state.items.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        state.last_indexed_at = Some(current_timestamp());

        let mut summary = ScanSummary {
            indexed: state.items.len(),
            scanned: Some(scanned_count),
            cached: Some(cached_count),
            errors,
            last_indexed_at: state.last_indexed_at,
        };
        if let Some(message) = ocr_status_message() {
            if !summary.errors.iter().any(|existing| existing == &message) {
                summary.errors.push(message);
            }
        }

        state.warnings = summary.errors.clone();
        let persist_result = persist_state(&self.storage_path, &state);
        self.emit_scan_progress(None, None, None);
        persist_result?;

        Ok(summary)
    }

    pub fn update_directories(&self, directories: Vec<String>) -> Result<ScanSummary> {
        println!("update_directories called with: {:?}", directories);
        
        let mut seen = std::collections::HashSet::new();
        let mut sanitised: Vec<String> = Vec::new();
        for dir in directories {
            let trimmed = dir.trim();
            if trimmed.is_empty() {
                continue;
            }
            let normalised = trimmed.to_string();
            if seen.insert(normalised.clone()) {
                sanitised.push(normalised);
            }
        }

        println!("Sanitised directories: {:?}", sanitised);

        let (last_indexed_at, item_count) = {
            let mut state = self.state.lock().expect("state poisoned");
            state.directories = sanitised.clone();
            println!("Saving directories to state: {:?}", state.directories);
            persist_state(&self.storage_path, &state)?;
            println!("Directories persisted successfully (no scan triggered)");
            (state.last_indexed_at, state.items.len())
        };

        // Return summary without scanning
        let mut summary = ScanSummary {
            indexed: item_count,
            scanned: None,
            cached: None,
            errors: Vec::new(),
            last_indexed_at,
        };
        
        if let Some(message) = ocr_status_message() {
            summary.errors.push(message);
        }
        
        Ok(summary)
    }

    pub fn rescan_directory(&self, directory: String) -> Result<ScanSummary> {
        let (target, existing_subset) = {
            let state = self.state.lock().expect("state poisoned");
            if let Some(target) = state
                .directories
                .iter()
                .find(|existing| *existing == &directory)
                .cloned()
            {
                let subset = state
                    .items
                    .iter()
                    .filter(|item| path_within(&item.path, &target))
                    .cloned()
                    .collect::<Vec<_>>();
                (Some(target), subset)
            } else {
                (None, Vec::new())
            }
        };

        let target = target
            .ok_or_else(|| AppError::Message(format!("Directory not linked: {directory}")))?;

        // Create callback that saves state after each indexed file
        let storage_path = self.storage_path.clone();
        let state_mutex = &self.state;
        
        let mut progress_cb = |path: &str, status: &str, debug: Option<&str>| self.emit_scan_progress(Some(path), Some(status), debug);
        
        let mut on_item_indexed = |item: crate::models::SlideIndexItem| {
            let mut state = state_mutex.lock().expect("state poisoned");
            // Add or update the item
            if let Some(pos) = state.items.iter().position(|i| i.path == item.path) {
                state.items[pos] = item;
            } else {
                state.items.push(item);
            }
            state.last_indexed_at = Some(current_timestamp());
            // Save immediately after each file
            if let Err(e) = persist_state(&storage_path, &state) {
                println!("⚠️  Failed to save cache after indexing file: {}", e);
            } else {
                println!("💾 Cache saved (items: {})", state.items.len());
            }
        };
        
        let outcome = scan_directories(&[target.clone()], &existing_subset, &mut progress_cb, &mut on_item_indexed);
        let ScanOutcome {
            items: new_items,
            errors,
            scanned_count,
            cached_count,
        } = match outcome {
            Ok(result) => result,
            Err(error) => {
                self.emit_scan_progress(None, None, None);
                return Err(error);
            }
        };

        let mut state = self.state.lock().expect("state poisoned");
        state.items.retain(|item| !path_within(&item.path, &target));
        state.items.extend(new_items);
        state.items.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        state.last_indexed_at = Some(current_timestamp());

        let mut summary = ScanSummary {
            indexed: state.items.len(),
            scanned: Some(scanned_count),
            cached: Some(cached_count),
            errors,
            last_indexed_at: state.last_indexed_at,
        };

        if let Some(message) = ocr_status_message() {
            if !summary.errors.iter().any(|existing| existing == &message) {
                summary.errors.push(message);
            }
        }

        state.warnings = summary.errors.clone();
        let persist_result = persist_state(&self.storage_path, &state);
        self.emit_scan_progress(None, None, None);
        persist_result?;

        Ok(summary)
    }

    pub fn search(&self, query: &str) -> SearchResponse {
        let state = self.state.lock().expect("state poisoned");
        let pattern = SearchPattern::new(query);
        let items = state
            .items
            .iter()
            .cloned()
            .filter(|item| matches_query(item, &pattern))
            .collect::<Vec<SlideIndexItem>>();
        SearchResponse {
            total: items.len(),
            items,
            last_indexed_at: state.last_indexed_at,
        }
    }

    pub fn find_item(&self, id: &str) -> Option<SlideIndexItem> {
        let state = self.state.lock().expect("state poisoned");
        state.items.iter().find(|item| item.id == id).cloned()
    }

    pub fn clear_cache(&self) -> Result<()> {
        let mut state = self.state.lock().expect("state poisoned");
        state.items.clear();
        state.last_indexed_at = Some(current_timestamp());
        state.warnings.clear();
        persist_state(&self.storage_path, &state)?;
        Ok(())
    }

    fn emit_scan_progress(&self, path: Option<&str>, status: Option<&str>, debug_info: Option<&str>) {
        let payload = ScanProgressPayload {
            path: path.map(|value| value.to_string()),
            status: status.map(|value| value.to_string()),
            debug_info: debug_info.map(|value| value.to_string()),
        };
        let _ = self.app_handle.emit_all("scan-progress", payload);
    }
}

fn load_state(path: &Path) -> Result<AppState> {
    let raw = fs::read_to_string(path)?;
    let parsed: AppState = serde_json::from_str(&raw)?;
    Ok(parsed)
}

fn persist_state(path: &Path, state: &AppState) -> Result<()> {
    let payload = serde_json::to_string_pretty(state)?;
    fs::write(path, payload)?;
    Ok(())
}

fn path_within(path: &str, directory: &str) -> bool {
    let file_path = Path::new(path);
    let dir_path = Path::new(directory);
    file_path == dir_path || file_path.starts_with(dir_path)
}
