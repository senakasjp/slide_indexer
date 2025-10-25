# Changelog

All notable changes to Slides Indexer will be documented in this file.

## [0.4.1] - 2025-10-25

### 📊 Cache Statistics Display
- **Directory Cache Info**
  - Shows word count for each linked directory (snippets + slides + keywords)
  - Displays cache size in appropriate units (B, KB, MB, GB)
  - Updates automatically when rescanning or linking folders
  - Formatted with icons for easy readability
  
- **Visual Display**
  - 📝 Word count icon with formatted numbers (e.g., "1,495,503 words")
  - 💾 Database icon with file size (e.g., "9.56 MB")
  - Information displayed below directory path in directory list
  
### 🐛 Bug Fixes
- **Directory Path Matching**
  - Fixed issue where directories with spaces (e.g., "Calibre Library") weren't matching items correctly
  - Improved path comparison logic to handle both absolute paths and directory names
  - Applied consistent matching logic across stats calculation and search filtering
  
- **Search Filtering**
  - Fixed search results not showing for directories with spaces or special characters
  - Both local (offline) and remote search now use robust directory matching
  - Search results now properly filter when specific directories are selected

### 🔧 Technical Changes
- Added `getDirectoryStats()` function to calculate per-directory metrics
- Updated `filterLocalItems()` to use improved path matching
- Updated `executeRemoteSearch()` to use improved path matching
- Consistent directory name extraction across all path-matching operations

---

## [0.4.0] - 2025-10-24

### 💾 Incremental Cache Saving (Critical Fix)
- **Progressive Persistence**
  - **Problem**: Previously, cache was saved only at the END of entire scan
  - **Impact**: If scan interrupted (app closed, crash, timeout), ALL progress lost
  - **Real Issue**: Calibre Library PDFs with slow OCR never got cached because scans didn't complete
  - **Solution**: Cache now saves after EACH file is successfully indexed
  - **Result**: No lost progress, interrupted scans don't waste work
  
- **Implementation**
  - Added `on_item_indexed` callback to `scan_directories()` function
  - Callback fires immediately after each file is processed
  - Updates in-memory state and persists to `index.json` on disk
  - Console logs: "💾 Cache saved (items: X)" after each save
  - Works for PPTX, PPT, and PDF files
  
- **Benefits for Long OCR Scans**
  - Scanned PDFs requiring OCR (slow process) are now cached immediately
  - Can stop/restart scans without losing OCR work
  - Safe from crashes, force-quits, and system issues
  - Progress is preserved even if only partial scan completes

### 🔧 Technical Changes
- Modified `StateManager::rescan()` to create `on_item_indexed` callback
- Modified `StateManager::rescan_directory()` to use same callback
- Updated `scan_directories()` signature to accept callback parameter
- Callback adds/updates item in state and calls `persist_state()` immediately
- All three file types (PPTX, PPT, PDF) call callback after successful indexing

### 📊 User Experience
- More verbose console output with cache save confirmations
- Visible progress indicator (item count increases with each save)
- Can safely interrupt scans knowing work is saved
- Subsequent scans will show already-indexed files as "Cached"

### ⚠️ Breaking Changes
- None for users - internal API change only
- `scan_directories()` function signature updated (internal only)

---

## [0.3.2] - 2025-10-24

### 🔍 Combined Scan Details + OCR Status
- **Fixed**: OCR status was replacing scan details panel
- **Now**: Shows BOTH scan reason AND OCR status together
- Visual separator (dashed line) between sections
- Users can see WHY file is being scanned WHILE OCR runs
- Example: "New File Detected" + checksum info + "OCR Processing" message

---

## [0.3.1] - 2025-10-24

### 🔍 Enhanced OCR Visibility
- **OCR Status in Scan Details Panel**
  - Scan details panel now shows OCR processing status
  - Purple-themed UI when OCR is running
  - Messages: "Extracting text from images..." and "This may take a few moments"
  - Visual feedback with icons: 🔍 magnifying glass, 🖼️ image, ⏳ hourglass
  - Panel header changes to "OCR Processing" during OCR operations
  
### 🎨 UI Polish
- Dynamic panel colors: blue for normal scans, purple for OCR
- Better icon selection for different scan states
- More user-friendly messaging

---

## [0.3.0] - 2025-10-24

### 📊 Live Scan Details Panel (Major Feature)
- **Real-Time Visual Feedback**
  - Beautiful scan details panel shows for ALL files being scanned
  - No longer just for debugging - permanent feature of the application
  - Shows detailed information: checksums, modification times, file status
  - Color-coded icons for instant understanding
  
- **Visual Information Displayed**
  - ➕ New files: Shows "New File Detected" with first-time indexing info
  - 📊 Rescanned files: Shows old vs. new checksums and mod times
  - ✅ Checksum matches: Explains why file is being rescanned despite content unchanged
  - ❌ Content changed: Shows checksum mismatch with old/new values
  - 🕐 Modification times: Displays both cached and current timestamps
  
- **Enhanced User Experience**
  - Gradient border (blue for scans, purple for OCR)
  - Icon-based information layout
  - Clear, concise messaging
  - Works for PPTX, PPT, and PDF files

### ⚡ Performance Optimizations
- **Two-Tier Caching Strategy**
  - **Quick check**: Compare modification time only (instant, no I/O)
  - **Full check**: Calculate checksum only if mod time changed
  - Result: Dramatically faster rescans when files haven't changed
  - Console shows: "✓ Cached (quick)" vs "✓ Cached (checksum)"
  
- **Optimized Checksum Calculation**
  - Only calculate checksums for changed or new files
  - Skip expensive SHA-256 calculation when modification time matches
  - Reduces rescan time from 2-3 seconds to <0.1 seconds for unchanged files

### 🗑️ Automatic Cache Cleanup
- **Deleted File Detection**
  - Tracks all files found during scan
  - Automatically removes deleted files from cache
  - Shows in console: "🗑️ Removed from cache (deleted): filename.pdf"
  - Scan summary includes: "Removed: X (deleted files)"
  
- **Benefits**
  - Keeps cache relevant and accurate
  - Prevents stale entries from accumulating
  - Automatic maintenance without user intervention

### 🐛 Enhanced Debugging
- **Comprehensive Path Logging**
  - Shows scan initialization with cache size and directories
  - Displays exact file paths being looked up
  - Searches for similar paths in cache to detect path mismatches
  - Detailed modification time comparisons
  
- **Problem File Tracking**
  - Specific debugging for problematic files (configurable in code)
  - Shows cache lookup results and similar paths
  - Explains caching decisions in detail

### 🔧 Technical Improvements
- Removed unused `should_reuse_existing()` function
- Cleaner caching logic inline with scan loops
- Better separation of concerns: quick check → checksum check → scan
- Improved error messages and logging

---

## [0.1.9] - 2025-10-24

### 🐛 Critical Bug Fix
- **Fixed: Empty Content Caching Issue**
  - **Problem**: Files with no extractable text (scanned PDFs, encrypted PDFs, image-only PDFs) were being re-scanned on every rescan, even when checksums matched perfectly
  - **Root Cause**: `should_reuse_existing()` function in `scanner.rs` required files to have `snippet` or `keywords` content, even when SHA-256 checksums confirmed the file was unchanged
  - **Impact**: Large document collections with many scanned PDFs experienced slow rescans (100+ files re-scanned unnecessarily)
  - **Solution**: Modified caching logic to trust checksum comparison as the primary indicator
    - If checksums match → always cache (regardless of content)
    - If checksums unavailable → fall back to modification time + content check
  - **Result**: Rescan performance dramatically improved for collections with non-text PDFs

### 📊 Debug & Monitoring Improvements
- **Visible Debug Logging in UI**
  - Added yellow debug info box that appears in the UI (not just console logs)
  - Shows real-time checksum comparison, modification times, and caching decisions
  - Helpful for troubleshooting file scanning issues in production
  - Debug info automatically displays for problematic files
  
- **Enhanced Scan Status Indicators**
  - ✓ Green checkmark for cached files
  - ✗ Orange cross for files being re-scanned
  - Visual feedback shows which files are being processed
  
- **Detailed Diagnostic Information**
  - Shows checksum match/mismatch status
  - Displays modification time comparisons
  - Explains why files are being cached or re-scanned
  - Indicates if files have extractable content

### 🔧 Technical Details
- **Modified Function**: `should_reuse_existing()` in `src-tauri/src/scanner.rs` (lines 1334-1353)
- **Key Change**: Line 1338-1339 now returns `true` unconditionally when checksums match
- **Streaming Checksums**: 8KB buffer for reliable SHA-256 calculation on large files
- **IPC Updates**: Added `debug_info` field to `ScanProgressPayload` for UI visibility

### 📝 Documentation Updates
- Added troubleshooting section for caching issues
- Documented the fix in README with technical explanation
- Updated caching logic documentation with rationale
- Added common issues guide for future reference

### 🧪 Testing Notes
- Tested with large PDF files (>50MB) with no extractable text
- Verified checksums persist correctly across rescans
- Confirmed "Scanned: 0, Cached: 100" behavior after initial scan
- Debug UI successfully displays diagnostic information

---

## [0.1.0] - 2025-10-24

### 🚀 Major Changes
- **Tauri-Only Application**: Converted to native desktop app
  - Removed Express server and Node.js backend
  - Single binary with Rust backend for better performance
  - Smaller download size (~10MB vs previous setup)
  - Faster startup and improved responsiveness

### ✨ New Features
- **Checksum-Based Caching**: SHA-256 checksums prevent unnecessary re-scanning
  - Scanned PDFs are cached permanently - OCR runs only once per file
  - Unchanged files skip processing entirely for dramatically faster rescans
  - Checksums persist across app restarts
  - Two-tier verification: checksum (primary) + modification time (fallback)

- **Console Logging**: Real-time feedback during scanning operations
  - `✓ Cached: filename.pdf` - File loaded from cache
  - `⟳ Scanning: filename.pdf` - File being actively scanned
  - `⟳ Running OCR on PDF: filename.pdf` - OCR in progress
  - Run from terminal to see logs: `/Applications/Slides\ Indexer.app/Contents/MacOS/Slides\ Indexer`

### 🎯 Search & UI Improvements
- **Search Highlighting**: All search terms highlighted in yellow throughout preview text
- **Directory Filtering**: Checkbox controls for selective folder searching
- **Manual Scanning**: Linking folders no longer triggers automatic indexing
- **Clear Cache Button**: One-click cache clearing for fresh re-indexing

### 🛠️ Technical Improvements
- **Rust Scanner**: High-performance file processing in Rust
- **Native File Dialogs**: System-native folder picker
- **Type-Safe IPC**: Tauri's command system for frontend-backend communication
- **Persistent Storage**: Index stored in user's application data directory

### 📦 Removed
- Express server and HTTP API
- Node.js backend scanner
- Browser-only offline mode
- PDF.js web worker (not needed for native app)

### 🐛 Bug Fixes
- Fixed checksum field initialization in Rust scanner
- Improved error handling for missing OCR dependencies
- Better state management across app restarts

### 📝 Documentation
- Updated README with Tauri-only instructions
- Added console logging documentation
- Clarified OCR dependency requirements
- Added build and development instructions

---

## Development Notes

### Building (v0.4.0)

```bash
# Install dependencies
npm install
brew install poppler tesseract  # For OCR support

# Debug build (recommended for development)
npm run tauri build -- --debug
# Output: src-tauri/target/debug/bundle/dmg/Slides Indexer_0.4.1_aarch64.dmg

# Release build (optimized)
npm run tauri build
# Output: src-tauri/target/release/bundle/dmg/Slides Indexer_0.4.1_aarch64.dmg
```

### Testing & Debugging

**This is a Tauri application** - all debugging is done via Terminal console, not browser console.

```bash
# Development mode (hot reload, console output, DevTools)
npm run tauri:dev

# Run installed debug build with console
./src-tauri/target/debug/bundle/macos/Slides\ Indexer.app/Contents/MacOS/Slides\ Indexer

# Run release build with console
/Applications/Slides\ Indexer.app/Contents/MacOS/Slides\ Indexer
```

### Cache Location
```
~/Library/Application Support/com.example.slidesindexer/slides-indexer/index.json
```

### Key Commands for Troubleshooting
- Watch for `💾 Cache saved (items: X)` - confirms incremental saves
- Look for `✓ Cached (quick)` vs `⟳ Scanning` - shows caching efficiency
- Check `🗑️ Removed from cache` - deleted file cleanup
- Monitor `⟳ Running OCR on PDF` - OCR operations

---

**Current Stable Version**: 0.4.1  
**Platform**: macOS (Apple Silicon & Intel)  
**Build Date**: October 25, 2025


