use std::{
    collections::{HashMap, HashSet},
    env,
    fs::{self, File},
    io::{Cursor, Read},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

use flate2::read::ZlibDecoder;
use globwalk::GlobWalkerBuilder;
use once_cell::sync::Lazy;
use regex::{escape, Regex, RegexBuilder};
use sha1::Sha1;
use sha2::{Digest, Sha256};
use tempfile::tempdir;
use zip::ZipArchive;

use crate::{
    error::{AppError, Result},
    models::{SlideIndexItem, SlideKind, SlidePreview},
};

const PPTX_GLOB: &str = "**/*.pptx";
const PPT_GLOB: &str = "**/*.ppt";
const PDF_GLOB: &str = "**/*.pdf";
const MAX_SNIPPET_LENGTH: usize = 240;
const MAX_KEYWORDS: usize = 40;
const MAX_OCR_PAGES: usize = 40;
const MIN_OCR_DPI: &str = "120";

static TEXT_RUN_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?s)<a:t[^>]*>(.*?)</a:t>").expect("valid regex"));
static PDF_TEXT_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\((?:\\.|[^\\)])*\)").expect("valid regex"));
static PDF_HEX_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"<([0-9A-Fa-f\s]+)>").expect("valid regex"));
static TOKEN_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"[a-z0-9]{3,}").expect("valid regex"));
static PAGE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"/Type\s*/Page\b").expect("valid regex"));

static NOISE_WORDS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    HashSet::from([
        "rectangle",
        "title",
        "subtitle",
        "body",
        "outline",
        "placeholder",
        "arial",
        "calibri",
        "bold",
        "italic",
        "regular",
    ])
});

static NOISE_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(r"^[a-zA-Z]{2}-[a-zA-Z]{2}$").expect("valid regex"),
        Regex::new(r"^latin-\d+$").expect("valid regex"),
        Regex::new(r"^slide\d*$").expect("valid regex"),
        Regex::new(r"^text\d*$").expect("valid regex"),
    ]
});

static SEARCH_TOKEN_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#""([^"]+)"|([^\s]+)"#).expect("valid regex"));

struct CommandPaths {
    pdftoppm: Option<PathBuf>,
    tesseract: Option<PathBuf>,
    pdftotext: Option<PathBuf>,
}

struct CommandStatus {
    paths: CommandPaths,
    missing: Vec<&'static str>,
}

static COMMAND_STATUS: Lazy<CommandStatus> = Lazy::new(resolve_command_status);

#[derive(Debug)]
pub struct ScanOutcome {
    pub items: Vec<SlideIndexItem>,
    pub errors: Vec<String>,
    pub scanned_count: usize,
    pub cached_count: usize,
}

#[derive(Debug)]
pub struct SearchPattern {
    terms: Vec<String>,
    phrases: Vec<String>,
    wildcards: Vec<Regex>,
    is_empty: bool,
}

impl SearchPattern {
    pub fn new(raw: &str) -> Self {
        let mut terms = Vec::new();
        let mut phrases = Vec::new();
        let mut wildcards = Vec::new();

        for capture in SEARCH_TOKEN_REGEX.captures_iter(raw) {
            if let Some(phrase) = capture.get(1) {
                let value = phrase.as_str().trim().to_lowercase();
                if !value.is_empty() {
                    phrases.push(value);
                }
            } else if let Some(token) = capture.get(2) {
                let value = token.as_str().trim();
                if value.is_empty() {
                    continue;
                }
                if value.contains('*') || value.contains('?') {
                    if let Some(regex) = wildcard_to_regex(value) {
                        wildcards.push(regex);
                    }
                } else {
                    terms.push(value.to_lowercase());
                }
            }
        }

        let is_empty = terms.is_empty() && phrases.is_empty() && wildcards.is_empty();
        Self {
            terms,
            phrases,
            wildcards,
            is_empty,
        }
    }
}

pub fn scan_directories(
    directories: &[String],
    existing: &[SlideIndexItem],
    progress: &mut dyn FnMut(&str, &str, Option<&str>),
    on_item_indexed: &mut dyn FnMut(SlideIndexItem),
) -> Result<ScanOutcome> {
    let mut aggregated = Vec::new();
    let mut errors = Vec::new();
    let mut existing_map: HashMap<String, SlideIndexItem> = HashMap::new();
    let mut scanned_count = 0;
    let mut cached_count = 0;
    let mut found_files: HashSet<String> = HashSet::new();
    
    // Build map of existing items
    for item in existing {
        existing_map.insert(item.path.clone(), item.clone());
    }
    
    println!("\n📊 Scan initialized:");
    println!("  Existing cached items: {}", existing_map.len());
    println!("  Directories to scan: {}", directories.len());
    for dir in directories {
        println!("    - {}", dir);
    }
    println!();

    for directory in directories {
        let path = Path::new(directory);
        if !path.exists() {
            errors.push(format!("Directory not found: {directory}"));
            continue;
        }

        let pptx_files = GlobWalkerBuilder::from_patterns(path, &[PPTX_GLOB])
            .max_depth(usize::MAX)
            .case_insensitive(true)
            .build()
            .map_err(|err| AppError::Message(err.to_string()))?;

        for entry in pptx_files.filter_map(|entry| entry.ok()) {
            let file_path = entry.path().to_path_buf();
            if is_temporary_deck(&file_path) {
                continue;
            }
            
            // Track this file was found
            found_files.insert(file_path.to_string_lossy().to_string());
            
            let modified_at = file_modified_ms(&file_path);
            
            // Quick check: if mod time unchanged, cache without calculating checksum
            if let Some(existing) = existing_map.get(file_path.to_string_lossy().as_ref()) {
                if let Some(mod_time) = modified_at {
                    if existing.updated_at == mod_time {
                        println!("✓ Cached (quick): {}", file_path.file_name().unwrap_or_default().to_string_lossy());
                        let path_string = file_path.to_string_lossy().to_string();
                        progress(&path_string, "cached", None);
                        aggregated.push(existing.clone());
                        cached_count += 1;
                        continue;
                    }
                }
            }
            
            // File changed or new - calculate checksum
            let checksum = match calculate_file_checksum(&file_path) {
                Ok(sum) => Some(sum),
                Err(err) => {
                    println!("⚠ Checksum failed for {}: {}", 
                        file_path.file_name().unwrap_or_default().to_string_lossy(), err);
                    None
                }
            };
            
            // Check if checksum matches (content unchanged despite time change)
            if let Some(existing) = existing_map.get(file_path.to_string_lossy().as_ref()) {
                if let (Some(existing_checksum), Some(new_checksum)) = (&existing.checksum, &checksum) {
                    if existing_checksum == new_checksum {
                        println!("✓ Cached (checksum): {}", file_path.file_name().unwrap_or_default().to_string_lossy());
                        let path_string = file_path.to_string_lossy().to_string();
                        progress(&path_string, "cached", None);
                        aggregated.push(existing.clone());
                        cached_count += 1;
                        continue;
                    }
                }
                
                let reason = match (&existing.checksum, &checksum) {
                    (None, None) => "both checksums missing".to_string(),
                    (None, Some(_)) => "existing has no checksum".to_string(),
                    (Some(_), None) => "new checksum failed to calculate".to_string(),
                    (Some(old), Some(new)) => format!("checksum changed: {}.. -> {}..", &old[..8], &new[..8]),
                };
                println!("⟳ Re-scanning (changed): {} - {}", 
                    file_path.file_name().unwrap_or_default().to_string_lossy(),
                    reason
                );
            }
            
            // Generate scan details for PPTX files being scanned
            let mut msg = String::new();
            let lookup_key = file_path.to_string_lossy();
            
            if let Some(existing) = existing_map.get(lookup_key.as_ref()) {
                msg.push_str("📊 Rescan Information:\n");
                msg.push_str(&format!("File content changed\n"));
                if let (Some(old), Some(new)) = (&existing.checksum, &checksum) {
                    if old != new {
                        msg.push_str(&format!("Old checksum: {:?}\n", &old[..8]));
                        msg.push_str(&format!("New checksum: {:?}", &new[..8]));
                    }
                }
            } else {
                msg.push_str("➕ New File Detected\n");
                msg.push_str("First time indexing this file");
            }
            
            let path_string = file_path.to_string_lossy().to_string();
            progress(&path_string, "scanning", Some(&msg));
            match index_pptx(&file_path, modified_at, checksum) {
                Ok(item) => {
                    on_item_indexed(item.clone());
                    aggregated.push(item);
                    scanned_count += 1;
                }
                Err(error) => errors.push(format!(
                    "Failed to index PPTX {}: {}",
                    file_path.display(),
                    error
                )),
            }
        }

        let ppt_files = GlobWalkerBuilder::from_patterns(path, &[PPT_GLOB])
            .max_depth(usize::MAX)
            .case_insensitive(true)
            .build()
            .map_err(|err| AppError::Message(err.to_string()))?;

        for entry in ppt_files.filter_map(|entry| entry.ok()) {
            let file_path = entry.path().to_path_buf();
            if is_temporary_deck(&file_path) {
                continue;
            }
            
            // Track this file was found
            found_files.insert(file_path.to_string_lossy().to_string());
            
            let modified_at = file_modified_ms(&file_path);
            
            // Quick check: if mod time unchanged, cache without calculating checksum
            if let Some(existing) = existing_map.get(file_path.to_string_lossy().as_ref()) {
                if let Some(mod_time) = modified_at {
                    if existing.updated_at == mod_time {
                        println!("✓ Cached (quick): {}", file_path.file_name().unwrap_or_default().to_string_lossy());
                        let path_string = file_path.to_string_lossy().to_string();
                        progress(&path_string, "cached", None);
                        aggregated.push(existing.clone());
                        cached_count += 1;
                        continue;
                    }
                }
            }
            
            // File changed or new - calculate checksum
            let checksum = match calculate_file_checksum(&file_path) {
                Ok(sum) => Some(sum),
                Err(err) => {
                    println!("⚠ Checksum failed for {}: {}", 
                        file_path.file_name().unwrap_or_default().to_string_lossy(), err);
                    None
                }
            };
            
            // Check if checksum matches (content unchanged despite time change)
            if let Some(existing) = existing_map.get(file_path.to_string_lossy().as_ref()) {
                if let (Some(existing_checksum), Some(new_checksum)) = (&existing.checksum, &checksum) {
                    if existing_checksum == new_checksum {
                        println!("✓ Cached (checksum): {}", file_path.file_name().unwrap_or_default().to_string_lossy());
                        let path_string = file_path.to_string_lossy().to_string();
                        progress(&path_string, "cached", None);
                        aggregated.push(existing.clone());
                        cached_count += 1;
                        continue;
                    }
                }
                
                let reason = match (&existing.checksum, &checksum) {
                    (None, None) => "both checksums missing".to_string(),
                    (None, Some(_)) => "existing has no checksum".to_string(),
                    (Some(_), None) => "new checksum failed to calculate".to_string(),
                    (Some(old), Some(new)) => format!("checksum changed: {}.. -> {}..", &old[..8], &new[..8]),
                };
                println!("⟳ Re-scanning (changed): {} - {}", 
                    file_path.file_name().unwrap_or_default().to_string_lossy(),
                    reason
                );
            }
            
            // Generate scan details for PPT files being scanned
            let mut msg = String::new();
            let lookup_key = file_path.to_string_lossy();
            
            if let Some(existing) = existing_map.get(lookup_key.as_ref()) {
                msg.push_str("📊 Rescan Information:\n");
                msg.push_str(&format!("File content changed\n"));
                if let (Some(old), Some(new)) = (&existing.checksum, &checksum) {
                    if old != new {
                        msg.push_str(&format!("Old checksum: {:?}\n", &old[..8]));
                        msg.push_str(&format!("New checksum: {:?}", &new[..8]));
                    }
                }
            } else {
                msg.push_str("➕ New File Detected\n");
                msg.push_str("First time indexing this file");
            }
            
            let path_string = file_path.to_string_lossy().to_string();
            progress(&path_string, "scanning", Some(&msg));
            match index_ppt(&file_path, modified_at, checksum) {
                Ok(item) => {
                    on_item_indexed(item.clone());
                    aggregated.push(item);
                    scanned_count += 1;
                }
                Err(error) => errors.push(format!(
                    "Failed to index PPT {}: {}",
                    file_path.display(),
                    error
                )),
            }
        }

        let pdf_files = GlobWalkerBuilder::from_patterns(path, &[PDF_GLOB])
            .max_depth(usize::MAX)
            .case_insensitive(true)
            .build()
            .map_err(|err| AppError::Message(err.to_string()))?;

        for entry in pdf_files.filter_map(|entry| entry.ok()) {
            let file_path = entry.path().to_path_buf();
            if is_temporary_deck(&file_path) {
                continue;
            }
            
            // Track this file was found
            found_files.insert(file_path.to_string_lossy().to_string());
            
            let is_problem_file = file_path.to_string_lossy().contains("LimanWu-NetworkSecurityAudit.pdf") ||
                                 file_path.to_string_lossy().contains("Data Communications and Networking With TC - Behrouz A. Forouzan.pdf");
            
            if is_problem_file {
                println!("\n=== DEBUG: Problem file detected ===");
                println!("Path: {}", file_path.display());
                println!("Path (as string): '{}'", file_path.to_string_lossy());
                println!("Is in existing_map: {}", existing_map.contains_key(file_path.to_string_lossy().as_ref()));
                
                // Show similar paths in existing_map
                println!("\n--- Searching existing_map for similar paths ---");
                let current_name = file_path.file_name().unwrap_or_default().to_string_lossy();
                for (cached_path, _) in &existing_map {
                    if cached_path.contains(&*current_name) || cached_path.contains("Calibre") {
                        println!("  Found similar: '{}'", cached_path);
                    }
                }
                println!("--- End similar paths ---\n");
            }
            
            let modified_at = file_modified_ms(&file_path);
            if is_problem_file {
                println!("Modified time: {:?}", modified_at);
            }
            
            // Step 1: Quick check - try to cache based on modification time ONLY (no checksum yet)
            if let Some(existing) = existing_map.get(file_path.to_string_lossy().as_ref()) {
                // If modification time unchanged, cache immediately without calculating checksum
                if let Some(mod_time) = modified_at {
                    if existing.updated_at == mod_time {
                        if is_problem_file {
                            println!("=== Quick cache: mod time unchanged ===");
                            println!("Existing: {}, Current: {}", existing.updated_at, mod_time);
                            println!();
                        }
                        println!("✓ Cached (quick): {}", file_path.file_name().unwrap_or_default().to_string_lossy());
                        let path_string = file_path.to_string_lossy().to_string();
                        progress(&path_string, "cached", None);
                        aggregated.push(existing.clone());
                        cached_count += 1;
                        continue;
                    } else if is_problem_file {
                        println!("⚠️ Modification time CHANGED!");
                        println!("  Existing: {}", existing.updated_at);
                        println!("  Current:  {}", mod_time);
                    }
                }
            }
            
            // Step 2: File changed or new - calculate checksum for verification
            let checksum = match calculate_file_checksum(&file_path) {
                Ok(sum) => {
                    if is_problem_file {
                        println!("Checksum calculated: {}...", &sum[..16]);
                    }
                    Some(sum)
                },
                Err(err) => {
                    println!("⚠ Checksum failed for {}: {}", 
                        file_path.file_name().unwrap_or_default().to_string_lossy(), err);
                    None
                }
            };
            
            // Step 3: Check if we can cache based on checksum match
            if let Some(existing) = existing_map.get(file_path.to_string_lossy().as_ref()) {
                if is_problem_file {
                    println!("Found in existing map:");
                    println!("  Existing checksum: {:?}", existing.checksum.as_ref().map(|s| &s[..16]));
                    println!("  Existing updated_at: {}", existing.updated_at);
                    println!("  Has content: snippet={}, keywords={}", 
                        !existing.snippet.trim().is_empty(), 
                        !existing.keywords.is_empty()
                    );
                }
                
                // Check if checksums match (content unchanged despite time change)
                if let (Some(existing_checksum), Some(new_checksum)) = (&existing.checksum, &checksum) {
                    if existing_checksum == new_checksum {
                        if is_problem_file {
                            println!("=== Cache: checksum match (time changed but content same) ===\n");
                        }
                        println!("✓ Cached (checksum): {}", file_path.file_name().unwrap_or_default().to_string_lossy());
                        let path_string = file_path.to_string_lossy().to_string();
                        progress(&path_string, "cached", None);
                        aggregated.push(existing.clone());
                        cached_count += 1;
                        continue;
                    }
                }
                
                // If we get here, file needs to be rescanned
                let reason = match (&existing.checksum, &checksum) {
                    (None, None) => "both checksums missing".to_string(),
                    (None, Some(_)) => "existing has no checksum".to_string(),
                    (Some(_), None) => "new checksum failed to calculate".to_string(),
                    (Some(old), Some(new)) => {
                        format!("checksum changed: {}.. -> {}..", &old[..8], &new[..8])
                    }
                };
                if is_problem_file {
                    println!("=== NOT Caching - will rescan ===");
                    println!("Reason: {}", reason);
                }
                println!("⟳ Re-scanning (changed): {} - {}", 
                    file_path.file_name().unwrap_or_default().to_string_lossy(),
                    reason
                );
            } else if is_problem_file {
                println!("NOT found in existing map - new file");
                println!("=== Will scan as new file ===\n");
            }
            
            let path_string = file_path.to_string_lossy().to_string();
            
            // Always generate debug info for files being scanned
            let mut msg = String::new();
            let lookup_key = file_path.to_string_lossy();
            
            if let Some(existing) = existing_map.get(lookup_key.as_ref()) {
                msg.push_str("📊 Rescan Information:\n");
                msg.push_str(&format!("Cached checksum: {:?}\n", existing.checksum.as_ref().map(|s| &s[..8])));
                msg.push_str(&format!("Current checksum: {:?}\n", checksum.as_ref().map(|s| &s[..8])));
                msg.push_str(&format!("Cached mod_time: {}\n", existing.updated_at));
                msg.push_str(&format!("Current mod_time: {:?}\n", modified_at));
                
                if let (Some(old), Some(new)) = (&existing.checksum, &checksum) {
                    if old == new {
                        msg.push_str("\n✅ Checksums MATCH\nContent unchanged, rescanning due to time change");
                    } else {
                        msg.push_str("\n❌ File content CHANGED\nChecksum mismatch detected");
                    }
                } else if existing.checksum.is_none() {
                    msg.push_str("\n⚠️ No cached checksum\nFirst scan or old cache format");
                }
            } else {
                msg.push_str("➕ New File Detected\n");
                msg.push_str(&format!("First time indexing this file\n"));
                msg.push_str(&format!("Current mod_time: {:?}\n", modified_at));
                msg.push_str(&format!("Checksum: {:?}", checksum.as_ref().map(|s| &s[..8])));
            }
            
            let debug_msg = Some(msg.clone());
            
            progress(&path_string, "scanning", debug_msg.as_deref());
            match index_pdf(&file_path, modified_at, checksum, progress, Some(msg)) {
                Ok(item) => {
                    on_item_indexed(item.clone());
                    aggregated.push(item);
                    scanned_count += 1;
                }
                Err(error) => errors.push(format!(
                    "Failed to index PDF {}: {}",
                    file_path.display(),
                    error
                )),
            }
        }
    }

    aggregated.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

    // Print scan summary
    let total = scanned_count + cached_count;
    // Clean up: Remove deleted files from cache
    let mut deleted_count = 0;
    for (cached_path, _) in &existing_map {
        if !found_files.contains(cached_path) {
            println!("🗑️  Removed from cache (deleted): {}", 
                Path::new(cached_path)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
            );
            deleted_count += 1;
        }
    }

    if total > 0 || deleted_count > 0 {
        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Scan Summary:");
        println!("  Total files:   {}", total);
        println!("  Scanned:       {} (newly processed)", scanned_count);
        println!("  Cached:        {} (skipped, unchanged)", cached_count);
        if deleted_count > 0 {
            println!("  Removed:       {} (deleted files)", deleted_count);
        }
        if !errors.is_empty() {
            println!("  Errors:        {}", errors.len());
        }
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    }

    Ok(ScanOutcome {
        items: aggregated,
        errors,
        scanned_count,
        cached_count,
    })
}

fn is_temporary_deck(path: &PathBuf) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.starts_with("~$"))
        .unwrap_or(false)
}

fn index_pptx(path: &PathBuf, modified_at: Option<u64>, checksum: Option<String>) -> Result<SlideIndexItem> {
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file)?;
    let mut slide_entries = Vec::new();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_string();
        if name.starts_with("ppt/slides/slide") && name.ends_with(".xml") {
            let mut xml = String::new();
            file.read_to_string(&mut xml)?;
            slide_entries.push(xml);
        }
    }

    let mut previews = Vec::new();
    let mut combined_text = String::new();
    for (index, xml) in slide_entries.into_iter().enumerate() {
        let runs = extract_text_runs(&xml);
        let stripped = strip_xml_tags(&runs);
        let sanitized = strip_binary_artifacts(&stripped);
        let filtered = filter_noise_tokens(&sanitized);
        let text = cleanup_whitespace(&filtered);
        if !text.is_empty() {
            previews.push(SlidePreview {
                index: index as u32 + 1,
                text: text.clone(),
            });
            if !combined_text.is_empty() {
                combined_text.push(' ');
            }
            combined_text.push_str(&text);
        }
    }

    let cleaned_text = cleanup_whitespace(&combined_text);
    let keywords = derive_keywords(&cleaned_text, &previews);

    Ok(SlideIndexItem {
        id: hash_of(path.to_string_lossy()),
        path: path.to_string_lossy().to_string(),
        name: path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| path.display().to_string()),
        kind: SlideKind::Pptx,
        slide_count: if previews.is_empty() {
            None
        } else {
            Some(previews.len() as u32)
        },
        snippet: truncate_snippet(&cleaned_text),
        keywords,
        updated_at: modified_at.unwrap_or_else(current_timestamp),
        slides: previews,
        checksum,
    })
}

fn index_pdf(
    path: &PathBuf, 
    modified_at: Option<u64>, 
    checksum: Option<String>,
    progress: &mut dyn FnMut(&str, &str, Option<&str>),
    initial_scan_details: Option<String>,
) -> Result<SlideIndexItem> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let contents = extract_pdf_contents(&buffer);
    let stripped = strip_xml_tags(&contents.text);
    let sanitized = strip_binary_artifacts(&stripped);
    let filtered = filter_noise_tokens(&sanitized);
    let cleaned = cleanup_whitespace(&filtered);

    let (mut previews, combined_from_pages) = build_previews_from_pages(&contents.pages);
    let mut snippet_source = if has_meaningful_text(&cleaned) {
        cleaned.clone()
    } else {
        String::new()
    };
    let mut keyword_source = if has_meaningful_text(&combined_from_pages) {
        combined_from_pages.clone()
    } else {
        String::new()
    };

    if !has_meaningful_text(&keyword_source) && has_meaningful_text(&snippet_source) {
        keyword_source = snippet_source.clone();
    }
    if !has_meaningful_text(&snippet_source) && has_meaningful_text(&keyword_source) {
        snippet_source = keyword_source.clone();
    }

    if (!has_meaningful_text(&snippet_source) || previews.is_empty())
        && COMMAND_STATUS.paths.pdftotext.is_some()
    {
        if let Ok(pdftotext_pages) = extract_pdf_with_pdftotext(path) {
            let (text_previews, combined) = build_previews_from_pages(&pdftotext_pages);
            if !text_previews.is_empty() {
                previews = text_previews;
            }
            if has_meaningful_text(&combined) {
                if !has_meaningful_text(&keyword_source) {
                    keyword_source = combined.clone();
                }
                if !has_meaningful_text(&snippet_source) {
                    snippet_source = combined.clone();
                }
            }
        }
    }

    if !has_meaningful_text(&snippet_source) || previews.is_empty() {
        println!("⟳ Running OCR on PDF: {}", path.file_name().unwrap_or_default().to_string_lossy());
        let path_string = path.to_string_lossy().to_string();
        
        // Combine initial scan details with OCR status
        let mut combined_msg = initial_scan_details.unwrap_or_default();
        if !combined_msg.is_empty() {
            combined_msg.push_str("\n\n━━━━━━━━━━━━━━━━━━━━━━\n\n");
        }
        combined_msg.push_str("🔍 OCR Processing:\nExtracting text from images...\nThis may take a few moments");
        
        progress(&path_string, "ocr", Some(&combined_msg));
        
        if let Ok(ocr_pages) = extract_pdf_with_ocr(path) {
            let (ocr_previews, combined) = build_previews_from_pages(&ocr_pages);
            if !ocr_previews.is_empty() {
                previews = ocr_previews;
            }
            if has_meaningful_text(&combined) {
                if !has_meaningful_text(&keyword_source) {
                    keyword_source = combined.clone();
                }
                if !has_meaningful_text(&snippet_source) {
                    snippet_source = combined.clone();
                }
            }
        }
    }

    if !has_meaningful_text(&keyword_source) && has_meaningful_text(&snippet_source) {
        keyword_source = snippet_source.clone();
    }

    let keywords = if has_meaningful_text(&keyword_source) {
        derive_keywords(&keyword_source, &previews)
    } else {
        Vec::new()
    };
    let snippet = truncate_snippet(&snippet_source);

    Ok(SlideIndexItem {
        id: hash_of(path.to_string_lossy()),
        path: path.to_string_lossy().to_string(),
        name: path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| path.display().to_string()),
        kind: SlideKind::Pdf,
        slide_count: contents.page_count.map(|value| value as u32),
        snippet,
        keywords,
        updated_at: modified_at.unwrap_or_else(current_timestamp),
        slides: previews,
        checksum,
    })
}

fn index_ppt(path: &PathBuf, modified_at: Option<u64>, checksum: Option<String>) -> Result<SlideIndexItem> {
    let buffer = fs::read(path)?;
    let ascii: String = buffer
        .iter()
        .map(|byte| match byte {
            0x09 | 0x0A | 0x0D => *byte as char,
            0x20..=0x7E => *byte as char,
            _ => ' ',
        })
        .collect();
    let cleaned = cleanup_whitespace(&filter_noise_tokens(&strip_binary_artifacts(
        &strip_xml_tags(&ascii),
    )));
    let previews = if cleaned.is_empty() || is_gibberish(&cleaned) {
        Vec::new()
    } else {
        vec![SlidePreview {
            index: 1,
            text: cleaned.clone(),
        }]
    };
    let effective_snippet = if previews.is_empty() {
        String::new()
    } else {
        cleaned.clone()
    };
    let keywords = derive_keywords(&effective_snippet, &previews);

    Ok(SlideIndexItem {
        id: hash_of(path.to_string_lossy()),
        path: path.to_string_lossy().to_string(),
        name: path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| path.display().to_string()),
        kind: SlideKind::Ppt,
        slide_count: None,
        snippet: truncate_snippet(&effective_snippet),
        keywords,
        updated_at: modified_at.unwrap_or_else(current_timestamp),
        slides: previews,
        checksum,
    })
}

struct PdfContents {
    text: String,
    page_count: Option<usize>,
    pages: Vec<String>,
}

fn extract_pdf_contents(buffer: &[u8]) -> PdfContents {
    let content = String::from_utf8_lossy(buffer);
    let page_count = {
        let count = PAGE_REGEX.find_iter(&content).count();
        if count == 0 {
            None
        } else {
            Some(count)
        }
    };

    let mut segments = Vec::new();
    let mut cursor = 0usize;

    while let Some(stream_pos) = find_subsequence(&buffer[cursor..], b"stream") {
        let absolute_stream_pos = cursor + stream_pos;
        let data_offset = absolute_stream_pos + "stream".len();

        let mut data_start = data_offset;
        while data_start < buffer.len()
            && (buffer[data_start] == b'\r' || buffer[data_start] == b'\n')
        {
            data_start += 1;
        }
        if data_start >= buffer.len() {
            break;
        }

        if let Some(end_pos) = find_subsequence(&buffer[data_start..], b"endstream") {
            let data_end = data_start + end_pos;
            let raw = &buffer[data_start..data_end];

            let header_start = absolute_stream_pos.saturating_sub(256);
            let header_slice = &buffer[header_start..absolute_stream_pos];
            let header = String::from_utf8_lossy(header_slice);
            let has_flate = header.contains("/FlateDecode");

            let decoded = if has_flate {
                match inflate_data(raw) {
                    Ok(decoded) => decoded,
                    Err(_) => raw.to_vec(),
                }
            } else {
                raw.to_vec()
            };

            let extracted = extract_text_from_pdf_stream(&decoded);
            if !extracted.is_empty() {
                segments.push(extracted);
            }

            cursor = data_end + "endstream".len();
        } else {
            break;
        }
    }

    PdfContents {
        text: segments.join(" "),
        page_count,
        pages: segments,
    }
}

fn extract_pdf_with_pdftotext(path: &Path) -> Result<Vec<String>> {
    let Some(pdftotext) = COMMAND_STATUS.paths.pdftotext.as_ref() else {
        return Ok(Vec::new());
    };

    let output = Command::new(pdftotext)
        .arg("-layout")
        .arg("-enc")
        .arg("UTF-8")
        .arg(path)
        .arg("-")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .map_err(|error| AppError::Message(error.to_string()))?;
    if !output.status.success() {
        return Ok(Vec::new());
    }

    let raw = String::from_utf8_lossy(&output.stdout);
    let pages = raw
        .split('\u{c}')
        .map(|segment| segment.trim())
        .filter(|segment| !segment.is_empty())
        .map(|segment| segment.to_string())
        .collect::<Vec<String>>();
    Ok(pages)
}

fn extract_pdf_with_ocr(path: &Path) -> Result<Vec<String>> {
    let commands = &COMMAND_STATUS.paths;
    let (Some(pdftoppm), Some(tesseract)) = (&commands.pdftoppm, &commands.tesseract) else {
        return Ok(Vec::new());
    };

    let temp_dir = tempdir().map_err(|error| AppError::Message(error.to_string()))?;
    let prefix = temp_dir.path().join("page");

    let status = Command::new(pdftoppm)
        .arg("-png")
        .arg("-r")
        .arg(MIN_OCR_DPI)
        .arg(path)
        .arg(prefix.as_os_str())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|error| AppError::Message(error.to_string()))?;
    if !status.success() {
        return Ok(Vec::new());
    }

    let mut images: Vec<PathBuf> = fs::read_dir(temp_dir.path())
        .map_err(|error| AppError::Message(error.to_string()))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("png"))
                .unwrap_or(false)
        })
        .collect();

    images.sort();

    let mut pages = Vec::new();
    for (index, image_path) in images.into_iter().enumerate() {
        if index >= MAX_OCR_PAGES {
            break;
        }
        let output = Command::new(tesseract)
            .arg(&image_path)
            .arg("stdout")
            .arg("-l")
            .arg("eng")
            .arg("--psm")
            .arg("6")
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .map_err(|error| AppError::Message(error.to_string()))?;
        if !output.status.success() {
            continue;
        }
        let text = String::from_utf8_lossy(&output.stdout).to_string();
        if text.trim().is_empty() {
            continue;
        }
        pages.push(text);
    }

    Ok(pages)
}

fn extract_text_from_pdf_stream(stream: &[u8]) -> String {
    let content = String::from_utf8_lossy(stream);
    let mut segments: Vec<String> = Vec::new();

    for mat in PDF_TEXT_REGEX.find_iter(&content) {
        let raw = mat.as_str();
        if raw.len() < 2 {
            continue;
        }
        let decoded = decode_pdf_string(&raw[1..raw.len() - 1]);
        if !decoded.is_empty() {
            segments.push(decoded);
        }
    }

    for caps in PDF_HEX_REGEX.captures_iter(&content) {
        if let Some(segment) = caps.get(1) {
            let decoded = decode_pdf_hex_string(segment.as_str());
            if !decoded.is_empty() {
                segments.push(decoded);
            }
        }
    }

    segments.join(" ")
}

fn decode_pdf_string(input: &str) -> String {
    let mut chars = input.chars().peekable();
    let mut result = String::new();

    while let Some(ch) = chars.next() {
        if ch != '\\' {
            result.push(ch);
            continue;
        }
        match chars.peek() {
            Some('n') => {
                result.push('\n');
                chars.next();
            }
            Some('r') => {
                result.push('\r');
                chars.next();
            }
            Some('t') => {
                result.push('\t');
                chars.next();
            }
            Some('b') => {
                result.push('\u{0008}');
                chars.next();
            }
            Some('f') => {
                result.push('\u{000C}');
                chars.next();
            }
            Some('(') => {
                result.push('(');
                chars.next();
            }
            Some(')') => {
                result.push(')');
                chars.next();
            }
            Some('\\') => {
                result.push('\\');
                chars.next();
            }
            Some(oct @ '0'..='7') => {
                let mut octal = String::new();
                octal.push(*oct);
                chars.next();
                for _ in 0..2 {
                    if let Some(next) = chars.peek() {
                        if next.is_ascii_digit() && *next < '8' {
                            octal.push(*next);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                }
                if let Ok(value) = u8::from_str_radix(&octal, 8) {
                    result.push(value as char);
                }
            }
            Some(other) => {
                result.push(*other);
                chars.next();
            }
            None => {}
        }
    }

    result
}

fn decode_pdf_hex_string(input: &str) -> String {
    let sanitized: String = input.chars().filter(|ch| !ch.is_whitespace()).collect();
    if sanitized.is_empty() {
        return String::new();
    }

    let mut bytes = Vec::new();
    let mut chars = sanitized.chars().peekable();
    while let Some(first) = chars.next() {
        let second = chars.peek().copied().unwrap_or('0');
        let pair = format!("{first}{second}");
        if let Ok(value) = u8::from_str_radix(&pair, 16) {
            bytes.push(value);
        }
        if chars.peek().is_some() {
            chars.next();
        }
    }

    decode_pdf_encoded_bytes(&bytes)
}

fn decode_pdf_encoded_bytes(bytes: &[u8]) -> String {
    if bytes.len() >= 2 {
        match (bytes[0], bytes[1]) {
            (0xFE, 0xFF) => {
                let units: Vec<u16> = bytes[2..]
                    .chunks(2)
                    .filter_map(|chunk| {
                        if chunk.len() == 2 {
                            Some(u16::from_be_bytes([chunk[0], chunk[1]]))
                        } else {
                            None
                        }
                    })
                    .collect();
                if let Ok(value) = String::from_utf16(&units) {
                    return value;
                }
            }
            (0xFF, 0xFE) => {
                let units: Vec<u16> = bytes[2..]
                    .chunks(2)
                    .filter_map(|chunk| {
                        if chunk.len() == 2 {
                            Some(u16::from_le_bytes([chunk[0], chunk[1]]))
                        } else {
                            None
                        }
                    })
                    .collect();
                if let Ok(value) = String::from_utf16(&units) {
                    return value;
                }
            }
            _ => {}
        }
    }
    match String::from_utf8(bytes.to_vec()) {
        Ok(value) => value,
        Err(_) => bytes.iter().map(|&byte| byte as char).collect::<String>(),
    }
}

fn extract_text_runs(xml: &str) -> String {
    TEXT_RUN_REGEX
        .captures_iter(xml)
        .filter_map(|capture| capture.get(1))
        .map(|segment| decode_xml(segment.as_str()))
        .filter(|segment| !segment.trim().is_empty())
        .map(|segment| segment.trim().to_string())
        .collect::<Vec<String>>()
        .join(" ")
}

fn decode_xml(input: &str) -> String {
    let mut output = input
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&apos;", "'");

    static DECIMAL_ENTITY: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"&#(\d+);").expect("valid regex"));
    static HEX_ENTITY: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"&#x([0-9a-fA-F]+);").expect("valid regex"));

    output = DECIMAL_ENTITY
        .replace_all(&output, |caps: &regex::Captures| {
            caps.get(1)
                .and_then(|m| m.as_str().parse::<u32>().ok())
                .and_then(char::from_u32)
                .map(|c| c.to_string())
                .unwrap_or_default()
        })
        .into_owned();

    output = HEX_ENTITY
        .replace_all(&output, |caps: &regex::Captures| {
            caps.get(1)
                .and_then(|m| u32::from_str_radix(m.as_str(), 16).ok())
                .and_then(char::from_u32)
                .map(|c| c.to_string())
                .unwrap_or_default()
        })
        .into_owned();

    output
}

fn strip_binary_artifacts(input: &str) -> String {
    input
        .chars()
        .map(|ch| {
            if ch == '\u{FFFD}' || (ch.is_control() && !matches!(ch, '\n' | '\r' | '\t')) {
                ' '
            } else {
                ch
            }
        })
        .collect()
}

fn strip_xml_tags(input: &str) -> String {
    static TAG_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"<[^>]+>").expect("valid regex"));
    TAG_REGEX.replace_all(input, " ").to_string()
}

fn cleanup_whitespace(input: &str) -> String {
    input.split_whitespace().collect::<Vec<&str>>().join(" ")
}

fn filter_noise_tokens(input: &str) -> String {
    input
        .split_whitespace()
        .filter(|token| !is_noise_token(token))
        .collect::<Vec<&str>>()
        .join(" ")
}

fn is_noise_token(token: &str) -> bool {
    let stripped = token.replace(['(', ')'], "");
    if !stripped.chars().any(|ch| ch.is_ascii_alphabetic()) {
        return true;
    }
    let lowered = stripped.to_lowercase();
    if NOISE_WORDS.contains(lowered.as_str()) {
        return true;
    }
    NOISE_PATTERNS
        .iter()
        .any(|pattern| pattern.is_match(&lowered))
}

fn derive_keywords(text: &str, slides: &[SlidePreview]) -> Vec<String> {
    let mut frequencies: HashMap<String, usize> = HashMap::new();
    for capture in TOKEN_REGEX.find_iter(&text.to_lowercase()) {
        let token = capture.as_str().to_string();
        *frequencies.entry(token).or_insert(0) += 1;
    }

    let mut slide_tokens: HashSet<String> = HashSet::new();
    for slide in slides {
        let lowered = slide.text.to_lowercase();
        for capture in TOKEN_REGEX.find_iter(&lowered) {
            slide_tokens.insert(capture.as_str().to_string());
        }
    }

    let mut items: Vec<(String, usize)> = frequencies
        .into_iter()
        .filter(|(token, _)| !slide_tokens.contains(token))
        .collect();
    items.sort_by(|a, b| b.1.cmp(&a.1));
    items
        .into_iter()
        .take(MAX_KEYWORDS)
        .map(|(token, _)| token)
        .collect()
}

fn build_previews_from_pages(raw_pages: &[String]) -> (Vec<SlidePreview>, String) {
    let mut previews = Vec::new();
    let mut combined = String::new();

    for (index, raw_page) in raw_pages.iter().enumerate() {
        let stripped_page = strip_xml_tags(raw_page);
        let sanitized = strip_binary_artifacts(&stripped_page);
        let filtered = filter_noise_tokens(&sanitized);
        let cleaned = cleanup_whitespace(&filtered);
        if !has_meaningful_text(&cleaned) {
            continue;
        }
        if !combined.is_empty() {
            combined.push(' ');
        }
        combined.push_str(&cleaned);
        previews.push(SlidePreview {
            index: index as u32 + 1,
            text: cleaned,
        });
    }

    (previews, combined)
}

fn has_meaningful_text(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return false;
    }
    if trimmed.len() < 12 {
        return trimmed.chars().any(|ch| ch.is_ascii_alphanumeric());
    }
    if is_gibberish(trimmed) {
        return false;
    }
    true
}

fn is_gibberish(text: &str) -> bool {
    let compact: String = text.chars().filter(|ch| !ch.is_whitespace()).collect();
    if compact.len() < 40 {
        return false;
    }
    let alpha = compact
        .chars()
        .filter(|ch| ch.is_ascii_alphabetic())
        .count();
    if alpha == 0 {
        return true;
    }
    let alpha_ratio = alpha as f64 / compact.len() as f64;
    if alpha_ratio < 0.35 {
        return true;
    }
    let upper = compact.chars().filter(|ch| ch.is_ascii_uppercase()).count();
    if alpha > 80 && (upper as f64 / alpha as f64) > 0.9 {
        return true;
    }
    let long_tokens = text
        .split_whitespace()
        .filter(|token| token.chars().count() > 40)
        .count();
    long_tokens > 2
}

fn file_modified_ms(path: &Path) -> Option<u64> {
    fs::metadata(path)
        .ok()
        .and_then(|meta| meta.modified().ok())
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis() as u64)
}

fn truncate_snippet(text: &str) -> String {
    if text.chars().count() <= MAX_SNIPPET_LENGTH {
        text.to_string()
    } else {
        text.chars().take(MAX_SNIPPET_LENGTH).collect()
    }
}

pub(crate) fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

fn hash_of(input: impl AsRef<str>) -> String {
    let mut hasher = Sha1::new();
    hasher.update(input.as_ref().as_bytes());
    hex::encode(hasher.finalize())
}

fn calculate_file_checksum(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192]; // 8KB buffer for streaming
    
    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    
    Ok(hex::encode(hasher.finalize()))
}

fn resolve_command_status() -> CommandStatus {
    let pdftoppm = resolve_command("pdftoppm");
    let tesseract = resolve_command("tesseract");
    let pdftotext = resolve_command("pdftotext");

    let mut missing: Vec<&'static str> = Vec::new();
    if pdftoppm.is_none() {
        missing.push("pdftoppm");
    }
    if tesseract.is_none() {
        missing.push("tesseract");
    }
    if pdftotext.is_none() {
        missing.push("pdftotext");
    }

    CommandStatus {
        paths: CommandPaths {
            pdftoppm,
            tesseract,
            pdftotext,
        },
        missing,
    }
}

fn resolve_command(command: &str) -> Option<PathBuf> {
    let mut search_dirs: Vec<PathBuf> = Vec::new();
    if let Some(path_var) = env::var_os("PATH") {
        search_dirs.extend(env::split_paths(&path_var));
    }

    search_dirs.extend(default_command_dirs());

    #[allow(unused_mut)]
    let mut candidate_names = vec![command.to_string()];
    #[cfg(target_os = "windows")]
    {
        if !command.ends_with(".exe") {
            candidate_names.push(format!("{command}.exe"));
        }
    }

    for dir in search_dirs {
        for name in &candidate_names {
            let candidate = dir.join(name);
            if is_executable_path(&candidate) {
                return Some(candidate);
            }
        }
    }

    None
}

fn default_command_dirs() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = Vec::new();

    #[cfg(target_os = "macos")]
    {
        dirs.extend(
            [
                "/opt/homebrew/bin",
                "/usr/local/bin",
                "/usr/bin",
                "/bin",
                "/opt/local/bin",
            ]
            .into_iter()
            .map(PathBuf::from),
        );
    }

    #[cfg(target_os = "linux")]
    {
        dirs.extend(
            ["/usr/local/bin", "/usr/bin", "/bin", "/snap/bin"]
                .into_iter()
                .map(PathBuf::from),
        );
    }

    #[cfg(target_os = "windows")]
    {
        dirs.extend(
            [
                r"C:\\Program Files\\Tesseract-OCR",
                r"C:\\Program Files (x86)\\Tesseract-OCR",
                r"C:\\Program Files\\poppler\\bin",
                r"C:\\Program Files (x86)\\poppler\\bin",
            ]
            .into_iter()
            .map(PathBuf::from),
        );
    }

    dirs
}

fn is_executable_path(path: &Path) -> bool {
    match fs::metadata(path) {
        Ok(metadata) => {
            if !metadata.is_file() {
                return false;
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                return metadata.permissions().mode() & 0o111 != 0;
            }
            #[cfg(not(unix))]
            {
                true
            }
        }
        Err(_) => false,
    }
}

#[allow(dead_code)]
pub fn is_ocr_available() -> bool {
    COMMAND_STATUS.paths.pdftoppm.is_some() && COMMAND_STATUS.paths.tesseract.is_some()
}

pub fn ocr_status_message() -> Option<String> {
    if COMMAND_STATUS.missing.is_empty() {
        None
    } else {
        Some(format!(
            "PDF extraction tools missing: {}. Install them to enable full PDF scanning (e.g. `brew install poppler tesseract`).",
            COMMAND_STATUS.missing.join(", ")
        ))
    }
}

fn inflate_data(data: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = ZlibDecoder::new(Cursor::new(data));
    let mut output = Vec::new();
    decoder
        .read_to_end(&mut output)
        .map_err(|error| AppError::Message(error.to_string()))?;
    Ok(output)
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn wildcard_to_regex(pattern: &str) -> Option<Regex> {
    let mut converted = String::new();
    for ch in pattern.chars() {
        match ch {
            '*' => converted.push_str(".*"),
            '?' => converted.push('.'),
            _ => converted.push_str(&escape(&ch.to_string())),
        }
    }
    if converted.is_empty() {
        return None;
    }
    let final_pattern = format!(".*{}.*", converted);
    RegexBuilder::new(&final_pattern)
        .case_insensitive(true)
        .dot_matches_new_line(true)
        .build()
        .ok()
}

fn build_search_corpus(item: &SlideIndexItem) -> String {
    let mut parts = Vec::new();
    parts.push(item.name.to_lowercase());
    parts.push(item.path.to_lowercase());
    if !item.snippet.is_empty() {
        parts.push(item.snippet.to_lowercase());
    }
    if !item.slides.is_empty() {
        parts.extend(item.slides.iter().map(|slide| slide.text.to_lowercase()));
    }
    if !item.keywords.is_empty() {
        parts.push(item.keywords.join(" ").to_lowercase());
    }
    parts.join(" ")
}

pub fn matches_query(item: &SlideIndexItem, pattern: &SearchPattern) -> bool {
    if pattern.is_empty {
        return true;
    }
    let corpus = build_search_corpus(item);
    for phrase in &pattern.phrases {
        if !corpus.contains(phrase) {
            return false;
        }
    }
    for term in &pattern.terms {
        if !corpus.contains(term) {
            return false;
        }
    }
    for wildcard in &pattern.wildcards {
        if !wildcard.is_match(&corpus) {
            return false;
        }
    }
    true
}
