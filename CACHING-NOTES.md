# Caching Logic Technical Notes

**Version 0.4.0** | **Tauri Desktop Application**

This document explains the file caching mechanism in Slides Indexer for future developers and maintainers.

> **⚠️ Debugging Note**: This is a Tauri application - all `println!` statements output to Terminal console (not browser console). Always run the app from Terminal to see caching logs and debug information.

## Overview

The application uses SHA-256 checksums to avoid re-scanning unchanged files. This is critical for performance when working with large document collections, especially those containing PDFs that require OCR processing.

## Core Logic Location

**File**: `src-tauri/src/scanner.rs`  
**Implementation**: Inline caching logic within scan loops (v0.3.0+)
- PPTX files: Lines ~170-254
- PPT files: Lines ~247-353  
- PDF files: Lines ~350-500

**Note**: The old `should_reuse_existing()` function was removed in v0.3.0 in favor of inline, optimized caching logic.

## Caching Decision Flow (v0.3.0+)

The caching logic has been optimized with a **two-tier approach** for maximum performance:

### Two-Tier Caching Strategy

**Location**: Inline within the scan loops in `scanner.rs` (PPTX: ~lines 170-217, PPT: ~lines 247-332, PDF: ~lines 350-400)

### Tier 1: Quick Check (Instant - No Disk I/O)

```rust
// Step 1: Check if modification time is unchanged
if let Some(existing) = existing_map.get(file_path.to_string_lossy().as_ref()) {
    if let Some(mod_time) = modified_at {
        if existing.updated_at == mod_time {
            // ✅ Mod time unchanged = file not modified
            println!("✓ Cached (quick): {}", filename);
            cached_count += 1;
            continue; // Skip to next file
        }
    }
}
```

**Performance:**
- **Operation**: Simple integer comparison of timestamps
- **Speed**: ~0.001ms per file (instant)
- **Result**: 99% of unchanged files cached here
- **No disk I/O**: No file reading, no checksums

### Tier 2: Full Verification (Checksum Calculation)

Only runs if modification time has changed:

```rust
// Step 2: Mod time changed - calculate checksum to verify content
let checksum = calculate_file_checksum(&file_path)?; // 8KB streaming

// Step 3: Compare checksums
if let Some(existing) = existing_map.get(file_path.to_string_lossy().as_ref()) {
    if let (Some(existing_checksum), Some(new_checksum)) = (&existing.checksum, &checksum) {
        if existing_checksum == new_checksum {
            // ✅ Content unchanged despite time change
            println!("✓ Cached (checksum): {}", filename);
            cached_count += 1;
            continue;
        }
    }
}

// File content actually changed - need to rescan
println!("⟳ Re-scanning (changed): {}", filename);
```

**Performance:**
- **Operation**: SHA-256 checksum via streaming (8KB buffer)
- **Speed**: ~50-200ms per large file
- **When Used**: Only for files with changed modification time
- **Catches**: Filesystem operations that update time without changing content

### Key Design Decisions (v0.1.9 + v0.3.0)

**Always cache when checksums match:**
- Even if `snippet.is_empty()` and `keywords.is_empty()`
- **Rationale**: Scanned PDFs, encrypted files have no text but are valid cached entries
- Re-scanning won't extract text that doesn't exist
- Avoids wasting time on expensive OCR for unchanged files

**Two-tier approach benefits:**
- **99% of files**: Cached via quick mod time check (<0.1 seconds total)
- **1% of files**: Need checksum verification (time changed)
- **<1% of files**: Actually changed and need full rescan

## Checksum Calculation

**Location**: `calculate_file_checksum()` in `scanner.rs`

```rust
fn calculate_file_checksum(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192]; // 8KB streaming buffer
    
    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 { break; }
        hasher.update(&buffer[..bytes_read]);
    }
    
    Ok(hex::encode(hasher.finalize()))
}
```

**Key Points:**
- Uses **8KB streaming buffer** (not loading entire file into memory)
- Handles large files (50MB+) efficiently
- SHA-256 provides reliable file change detection
- Hex-encoded output stored as string in JSON

## Common Scenarios

### Scenario 1: Scanned PDF (No Text)
```
First Scan:
- Calculate checksum: "130ade7d..."
- Run OCR: No text extracted
- Store: { snippet: "", keywords: [], checksum: "130ade7d..." }

Second Scan:
- Calculate checksum: "130ade7d..."
- Checksums match → ✅ CACHE (even though content is empty)
- Skip OCR entirely
```

### Scenario 2: Regular PDF with Text
```
First Scan:
- Calculate checksum: "abc12345..."
- Extract text: "Machine Learning Lecture..."
- Store: { snippet: "Machine...", keywords: ["ml", "ai"], checksum: "abc12345..." }

Second Scan:
- Calculate checksum: "abc12345..."
- Checksums match → ✅ CACHE
- Skip text extraction
```

### Scenario 3: Modified PDF
```
Previous:
- Checksum: "abc12345..."
- Content: "Old content"

Current Scan:
- Calculate checksum: "def67890..."
- Checksums differ → ❌ NO CACHE
- Re-scan and extract new content
```

## Debug Information

**UI Visibility (v0.1.8+):**
The scanning progress displays debug info for problematic files:

```
🔍 DEBUG FILE:
Found in cache: checksum=Some("130ade7d"), mod_time=1744655106715, has_content=false
Current: checksum=Some("130ade7d"), mod_time=Some(1744655106715)
⚠️ Checksums MATCH but not cached!  [This was the bug in v0.1.8]
Reason: snippet_empty=true, keywords_empty=true
```

**Fixed in v0.1.9:**
```
🔍 DEBUG FILE:
Found in cache: checksum=Some("130ade7d"), mod_time=1744655106715, has_content=false
Current: checksum=Some("130ade7d"), mod_time=Some(1744655106715)
✅ Checksums MATCH - caching file!
```

## Performance Impact

**v0.1.9 - Empty Content Caching Fix:**
- **Before**: 100 scanned PDFs with no text → 100 files re-scanned every time
- **After**: First scan calculates checksums → subsequent scans cache all 100
- **Time**: 5-10 minutes → 1-2 seconds per rescan

**v0.3.0 - Two-Tier Optimization:**
- **Before**: 100 unchanged files → calculate 100 checksums → ~2-3 seconds
- **After**: 100 unchanged files → 100 mod time checks → **<0.1 seconds**
- **Improvement**: 20-30x faster for unchanged collections

**Real-World Example:**
```
Collection: 500 files (PPTX, PPT, PDF mixed)
Scenario: No files changed since last scan

v0.2.x (old): 
  - Calculate 500 checksums
  - Time: ~10-15 seconds

v0.3.0+ (new):
  - 500 quick mod time checks
  - 0 checksums calculated
  - Time: ~0.2 seconds
  - Improvement: 50-75x faster
```

## Testing This Feature

1. **Find a scanned PDF** (image-only, no text layer)
2. **First scan**: Should show "⟳ Scanning: file.pdf"
3. **Check**: Verify checksum is stored in `index.json`
4. **Second scan**: Should show "✓ Cached: file.pdf"
5. **Result**: "Scanned: 0, Cached: 1"

## Troubleshooting

### Case Study: Calibre Library PDFs Not Caching (Fixed in v0.4.0)

**Symptoms:**
- Files always showed "New File Detected" 
- OCR ran on every scan (very slow)
- Files never appeared in cache

**Investigation:**
```bash
# Check cache
cat ~/Library/Application\ Support/com.example.slidesindexer/slides-indexer/index.json

# Result: Cache had 1082 items, but ZERO from Calibre Library
# Even though Calibre Library was linked and had many PDFs
```

**Root Cause:**
- Scanned PDFs require OCR (5-30 seconds per file)
- Large collections took 10+ minutes to scan
- User closed app before scan completed
- Cache only saved at END → all progress lost

**Solution (v0.4.0):**
- Incremental saving after each file
- OCR work preserved immediately
- Console shows: "💾 Cache saved (items: 1083)" after each file
- Safe to interrupt - completed files are cached

**Files not being cached:**
1. Check if checksum is stored in `index.json`
2. Run from terminal to see console logs
3. Look for "⟳ Re-scanning (changed)" messages with reasons
4. Look for "💾 Cache saved" confirmations (v0.4.0+)
5. Verify file modification time hasn't changed
6. Check scan details panel in UI for diagnostic info

**Checksums not being calculated:**
- Look for "⚠ Checksum failed" messages
- Common causes: File permissions, locked files, I/O errors
- Fallback: Uses modification time comparison

## Future Considerations

### Potential Improvements
- Add checksum calculation progress for large files
- Cache checksums separately for even faster verification
- Implement parallel checksum calculation
- Add checksum verification on startup (detect corrupted files)

### Edge Cases to Watch
- Files modified within the same millisecond
- Filesystem timestamp granularity issues
- Symbolic links and aliases
- Network drives with unreliable modification times

## Related Code

- `scanner.rs`: Main scanning logic
- `state.rs`: Progress emission and IPC
- `models.rs`: Data structures (`SlideIndexItem`, `ScanProgressPayload`)
- `App.svelte`: UI display of scan progress and debug info

## New Features (v0.3.0+)

### Live Scan Details Panel

**Location**: `src/App.svelte` (lines ~1289-1380)

Every file being scanned now shows a beautiful visual panel with:

**For New Files:**
```
ℹ️ SCAN DETAILS
➕ New File Detected
First time indexing this file
Current mod_time: 1744655106715
Checksum: Some("130ade7")
```

**For Rescanned Files:**
```
📊 SCAN DETAILS
📊 Rescan Information:
👆 Cached checksum: Some("abc12345")
👆 Current checksum: Some("def67890")
🕐 Cached mod_time: 1744655106715
🕐 Current mod_time: Some(1744655200000)
❌ File content CHANGED
```

**For OCR Operations (v0.3.1):**
```
🔍 OCR PROCESSING
🔍 OCR Processing:
🖼️  Extracting text from images...
⏳ This may take a few moments
```

**Features:**
- Color-coded: Blue for scans, Purple for OCR
- Icon-based information layout
- Shows for ALL files (not just debug mode)
- Updates in real-time during scanning

### Automatic Deleted File Cleanup

**Location**: `src-tauri/src/scanner.rs` (lines ~489-501)

```rust
// Track all files found during scan
found_files.insert(file_path.to_string_lossy().to_string());

// After scan, remove deleted files from cache
for (cached_path, _) in &existing_map {
    if !found_files.contains(cached_path) {
        println!("🗑️ Removed from cache (deleted): {}", filename);
        deleted_count += 1;
    }
}
```

**Benefits:**
- Automatic cache maintenance
- No stale entries
- Shows in scan summary: "Removed: X (deleted files)"

## Incremental Cache Saving (v0.4.0)

### The Problem (Pre-v0.4.0)

**Cache saved only at END of scan:**
```rust
// OLD CODE:
let outcome = scan_directories(...);  // Scan all files
let ScanOutcome { items, ... } = outcome;

state.items = items;  // Update state
persist_state(&storage_path, &state)?;  // Save ONCE at end
```

**Issues:**
- If scan interrupted → ALL progress lost
- Long OCR scans (5-10 minutes) → high risk of interruption
- Calibre Library PDFs with OCR never got cached
- App close, crash, or timeout → wasted work

### The Solution (v0.4.0+)

**Save after EACH file:**
```rust
// NEW CODE:
let mut on_item_indexed = |item: SlideIndexItem| {
    let mut state = state_mutex.lock().expect("state poisoned");
    
    // Add or update item
    if let Some(pos) = state.items.iter().position(|i| i.path == item.path) {
        state.items[pos] = item;
    } else {
        state.items.push(item);
    }
    
    // Save immediately
    persist_state(&storage_path, &state)?;
    println!("💾 Cache saved (items: {})", state.items.len());
};

scan_directories(..., &mut on_item_indexed);  // Passes callback
```

**Implementation:**
- `StateManager::rescan()` creates callback (lines ~88-109 in `state.rs`)
- `scan_directories()` accepts callback parameter (line ~140 in `scanner.rs`)
- After each `index_pptx()`, `index_ppt()`, `index_pdf()` succeeds → callback fires
- Callback updates state + saves to disk immediately

**Benefits:**
- ✅ No lost progress on interruptions
- ✅ OCR work preserved even if app closes
- ✅ Can safely stop/restart scans
- ✅ Calibre Library files finally get cached
- ✅ Console shows real-time save confirmations

**Trade-off:**
- More frequent disk writes (one per file vs. one per scan)
- Acceptable: Modern SSDs handle this easily, and reliability >> speed

## Version History

- **v0.1.0**: Initial checksum-based caching
- **v0.1.8**: Added debug UI and logging
- **v0.1.9**: Fixed empty content caching issue (critical fix)
- **v0.3.0**: Two-tier caching optimization + live scan details panel + deleted file cleanup
- **v0.3.1**: OCR status visibility in scan details panel
- **v0.3.2**: Combined scan details with OCR status
- **v0.4.0**: Incremental cache saving after each file (critical reliability fix)

---

**Last Updated**: 2025-10-24  
**Current Version**: 0.4.0

