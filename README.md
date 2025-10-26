# Slides Indexer

**Version 0.4.3** - Native macOS desktop app for cataloging PowerPoint and PDF slide decks. Built with Tauri (Rust) + Svelte + Flowbite. Index, search, and preview presentation files with intelligent checksum-based caching, OCR support, and automatic document type detection.

> **⚠️ Tauri Desktop Application**: This is a **native desktop app** (not a web app). All debugging and troubleshooting should be done by running the app from Terminal to see console logs. There is no browser console - all logs appear in Terminal where you launch the app.

## 🚀 Quick Start

> **📖 For a condensed reference guide, see [QUICK-REFERENCE.md](./QUICK-REFERENCE.md)**

**For Users:**
1. Install the DMG: `Slides Indexer_0.4.3_aarch64.dmg`
2. Run from Terminal for logs: `/Applications/Slides\ Indexer.app/Contents/MacOS/Slides\ Indexer`
3. Link folders, click "Rescan", and search your presentations!

**For Developers:**
```bash
npm install                  # Install dependencies
brew install poppler tesseract  # OCR support
npm run tauri:dev           # Development mode
npm run tauri build -- --debug  # Debug build with logs
```

**Cache Location:**
```
~/Library/Application Support/com.example.slidesindexer/slides-indexer/index.json
```

## ✨ Features

- **Native Desktop App**: Fast, lightweight Tauri application with full file system access
- **Smart Indexing**: Automatically scans PowerPoint (PPTX/PPT) and PDF files
- **Intelligent Caching**: SHA-256 checksum-based caching prevents unnecessary re-scanning of unchanged files
  - **Scanned PDFs are cached**: OCR runs only once per PDF - subsequent scans reuse cached results
  - **Automatic cleanup**: Deleted files removed from cache automatically
  - **Two-tier verification**: Quick mod time check first, then checksum for changed files
- **Cache Statistics** (v0.4.1+): View cache metrics for each directory
  - **Word count display**: See total words cached per directory
  - **Cache size display**: Shows storage used in B/KB/MB/GB format
  - **Real-time updates**: Stats refresh automatically on rescan
- **Document Type Detection** (v0.4.3): Smart classification of presentations vs books
  - **Automatic detection**: PDFs classified by page orientation (landscape = presentations, portrait = books)
  - **Visual badges**: 📊 Blue badge for presentations, 📚 Green badge for books
  - **Filter buttons**: "All", "Presentations", "Books" - work without searching
  - **PowerPoint files**: Always classified as presentations
  - **Smart terminology**: Shows "pages" for books, "slides" for presentations throughout UI
- **Native Desktop UI** (v0.4.2): Professional desktop application appearance
  - **Solid backgrounds**: Clean, non-translucent design for better readability
  - **Consistent styling**: All primary buttons use orange color scheme
  - **Fixed-width layout**: Optimized for desktop screens (no responsive design)
  - **Stop scan control**: Red stop button to halt active scans with visual feedback
  - **Processing timer**: See how long each file takes to process
  - **Version badge**: Prominent version display for easy verification
- **Live Scan Details**: Real-time visual feedback during scanning
  - **Beautiful UI panel**: Shows detailed information for every file being processed
  - **OCR status indicator**: Purple magnifying glass icon when OCR is running
  - **Checksum comparison**: See why files are being rescanned vs. cached
  - **Color-coded icons**: Instant visual understanding of scan status
- **Full-Text Search**: Real-time search with phrase matching, wildcards, and keyword extraction
- **Search Highlighting**: All search terms are highlighted in yellow within preview text for easy identification
- **Directory Filtering**: Select which folders to include in search results with checkbox controls
- **OCR Support**: PDF text extraction with Tesseract and pdftotext integration for scanned documents
- **Slide Previews**: Instant slide previews with highlighted search matches
- **Cache Management**: Clear cache functionality for fresh re-indexing when needed
- **Modern UI**: Clean, responsive interface with dark/light theme support

## Installation

### Option 1: Download Pre-built App (Recommended)

Download the latest DMG from the releases:
```
Slides Indexer_0.4.3_aarch64.dmg
```

Install and run the app. The index is stored at:
```
~/Library/Application Support/com.example.slidesindexer/slides-indexer/index.json
```

**Note**: This is a **Tauri-only desktop application** (not a web app). All troubleshooting and debugging should be done via the Tauri app, not a browser.

### Option 2: Build from Source

**Prerequisites:**
- Node.js 18+ (for building frontend)
- Rust/Cargo (for building Tauri backend)
- macOS for building .app/.dmg (cross-platform support planned)

**For full PDF OCR support, install external tools:**
```bash
# macOS
brew install poppler tesseract

# Ubuntu/Debian
sudo apt-get install poppler-utils tesseract-ocr

# Windows
# Download from official websites or use chocolatey
```

**Build steps:**
```bash
# Install dependencies
npm install

# Build the Tauri app and DMG
npm run tauri:build
```

The built app will be at:
```
src-tauri/target/release/bundle/macos/Slides Indexer.app
src-tauri/target/release/bundle/dmg/Slides Indexer_0.4.3_aarch64.dmg
```

**Debug build** (includes DevTools and verbose logging):
```
src-tauri/target/debug/bundle/macos/Slides Indexer.app
src-tauri/target/debug/bundle/dmg/Slides Indexer_0.4.3_aarch64.dmg
```

## Development

### Running in Development Mode

```bash
# Development mode with hot reload and console output
npm run tauri:dev
```

This starts Vite dev server and launches the Tauri app window with console logging enabled.

**Important for Tauri Development:**
- DevTools are available in debug builds (press `Cmd+Option+I` in dev mode)
- All `println!` statements from Rust appear in the Terminal
- Frontend logs (`console.log`) appear in DevTools console
- For troubleshooting, always check Terminal output first

### Debug vs Release Builds

```bash
# Debug build (verbose logging, DevTools enabled)
npm run tauri build -- --debug

# Release build (optimized, smaller size)
npm run tauri build
```

## 🚀 Usage

### Basic Workflow
1. **Link Folders**: Click "Link folder" to add directories containing your documents
2. **Manual Indexing**: Click "Rescan" button to scan and index all PowerPoint and PDF files
3. **Search**: Use the search bar to find documents by content, keywords, or phrases
4. **Preview**: Click any result to see page/slide previews with highlighted matches
5. **Open Files**: Click the file icon to open documents in their native applications

**Note**: Linking folders no longer triggers automatic scanning. Use the "Rescan" button when you're ready to index files.

### Advanced Features

#### Intelligent Caching
- **SHA-256 checksums** are automatically calculated for each indexed file
- **Unchanged files are skipped** during re-scans for dramatically faster performance
- **Scanned PDFs never re-run OCR** unless the file content actually changes
- **Incremental saving** - Cache saves after each file (v0.4.0+)
  - Safe to interrupt scans - already-indexed files are preserved
  - OCR work is never lost, even if app closes
  - Console shows: "💾 Cache saved (items: X)" after each file
- Caching uses two-tier verification:
  1. **Quick check**: Modification time comparison (instant)
  2. **Full check**: Checksum calculation only if time changed
- Use "Clear Cache" button to force a complete re-index when needed
- Checksums are stored persistently across sessions

#### Viewing Cache Status (Console Logs)
To see which files are being cached vs. scanned, run the app from terminal:

```bash
# For installed app
/Applications/Slides\ Indexer.app/Contents/MacOS/Slides\ Indexer

# For dev mode
npm run tauri:dev
```

During scanning, you'll see console output:
- `✓ Cached (quick): filename.pdf` - File loaded from cache via mod time check (instant)
- `✓ Cached (checksum): filename.pdf` - File cached via checksum match (time changed, content same)
- `⟳ Scanning: filename.pdf` - File being actively scanned
- `⟳ Running OCR on PDF: filename.pdf` - OCR in progress (first time only)
- `💾 Cache saved (items: 1083)` - Cache persisted to disk after each file (v0.4.0+)
- `🗑️ Removed from cache (deleted): filename.pdf` - Deleted file cleaned up

**First scan**: All files will show `⟳ Scanning:` as checksums are calculated, cache saves after each
**Subsequent scans**: Unchanged files show `✓ Cached (quick):` and process instantly

#### Search Capabilities
- **Exact Phrases**: Wrap text in quotes for exact phrase matching
- **Wildcards**: Use `*` for multiple characters, `?` for single characters
- **Real-time**: Results update instantly as you type
- **Keywords**: Automatically extracts and searches relevant keywords
- **Highlighted Matches**: Search terms are highlighted in yellow throughout preview text
- **Directory Filtering**: Checkbox controls to include/exclude specific folders from search

#### Directory Management
- **Selective Search**: Click checkboxes next to directories to filter search results
- **Select All/Deselect All**: Quick toggle button for bulk directory selection
- **Visual Indicators**: Badge shows how many folders are selected for search
- **Persistent Selection**: Directory preferences are maintained across searches

#### OCR Support
- Scanned PDFs are processed with OCR for text extraction
- Requires `poppler` and `tesseract` installation for full functionality
- Fallback text extraction works without external dependencies

### Troubleshooting

#### Common Issues

**Files always showing as "Scanned" instead of "Cached":**
- **Symptom**: Rescan shows non-zero scan count even when no files changed
- **Cause**: Files with no extractable content (scanned PDFs, encrypted PDFs) 
- **Fixed in v0.1.9**: Checksums now properly cache files regardless of content
- **Debug**: Scan details panel in UI shows why files aren't being cached

**Files showing as "New File Detected" every time:**
- **Symptom**: Same files appear as new on every scan, OCR runs repeatedly
- **Cause (Pre-v0.4.0)**: Cache was only saved at END of scan - interrupted scans lost all progress
- **Fixed in v0.4.0**: Cache saves after EACH file is indexed
- **Solution**: Let complete scan finish, or check console for "💾 Cache saved" confirmations
- **Check**: Look at `~/Library/Application Support/com.example.slidesindexer/slides-indexer/index.json`

**Large PDFs taking long to scan:**
- **First Scan**: OCR runs once per PDF (can take 5-30 seconds per file)
- **Subsequent Scans**: PDF should show "✓ Cached" and skip OCR
- **If not caching**: Check scan details panel - shows checksum and mod time info
- **v0.4.0+**: OCR work is saved immediately, safe to interrupt

**OCR not working:**
- Install required dependencies: `brew install poppler tesseract`
- App will show warning if dependencies missing
- Basic text extraction works without OCR

**Debugging & Console Logs:**
This is a **Tauri desktop application** - all debug information appears in the Terminal/console, not in a browser console.

**To see detailed logs**, always run the app from Terminal:
```bash
# For installed app
/Applications/Slides\ Indexer.app/Contents/MacOS/Slides\ Indexer

# For development
npm run tauri:dev
```

**What you'll see in console:**
- Scan initialization and cache stats
- File-by-file progress with status icons
- Checksum calculations and cache decisions
- OCR operations and progress
- Cache save confirmations: "💾 Cache saved (items: X)"
- Deleted file cleanup notifications

**In-App Help:**
Hit the "Help" button in the app header for a built-in usage guide and workflow tips.

## Available Scripts

- `npm run tauri:dev` – run app in development mode with hot reload
- `npm run tauri:build` – build production app and DMG
- `npm run build` – build frontend assets only
- `npm run dev` – start Vite dev server (for UI testing)
- `npm run check` – type-check Svelte + TypeScript
- `npm run lint` – optional Biome linting

## 🛠️ Technical Stack

### Frontend
- **[Svelte 4](https://svelte.dev/)** + **[Vite](https://vitejs.dev/)** - Modern reactive framework
- **[Tailwind CSS](https://tailwindcss.com/)** + **[Flowbite-Svelte](https://flowbite-svelte.com/)** - Utility-first styling and components
- **[Tauri API](https://tauri.app/)** - Native system integration and file dialogs

### Backend (Rust)
- **[Tauri](https://tauri.app/)** - Rust-based desktop app framework
- **File Processing**: High-performance scanning and indexing in Rust
- **Commands**: Exposed Rust functions callable from frontend

### File Processing (All in Rust Backend)
- **PowerPoint (PPTX)**: ZIP parsing with deflate decompression, XML text extraction
- **PowerPoint (PPT)**: Binary format ASCII text extraction
- **PDF**: Custom stream parsing with regex-based text extraction
- **OCR**: External tools (`pdftotext`, `pdftoppm`, `tesseract`) via Rust Command API
- **Caching**: SHA-256 checksums with 8KB streaming buffer, incremental persistence

### Data Storage
- **JSON** - Slide metadata and index storage in app data directory
- **Checksums** - SHA-256 file hashes for efficient caching
- **State Management** - Persistent application state managed by Rust backend

## 🔧 Architecture

Native desktop application built with:

1. **Tauri Framework**: Rust backend + Svelte frontend in a single native app
2. **Rust Scanner**: High-performance file indexing with checksum-based caching
3. **IPC Communication**: Frontend calls Rust commands via Tauri's type-safe IPC
4. **File System Access**: Full native file system access through Tauri APIs
5. **Local Storage**: Index stored in user's application data directory

## 🆕 Recent Updates

### v0.4.3 - Document Type Detection (Latest)
- **📚 Smart Classification**
  - Automatically distinguishes between presentations and books
  - PDF page orientation detection (landscape = presentations, portrait = books)
  - Perfect for separating Beamer slides from Calibre Library books
  - PowerPoint files (PPTX/PPT) always classified as presentations
  
- **🎯 Visual Indicators & Filtering**
  - 📊 Blue "Presentation" badges for slides and decks
  - 📚 Green "Book" badges for books and documents
  - Three filter buttons: "All", "Presentations", "Books"
  - Works immediately - no search required, click to see all documents of that type
  - Instantly filter search results by document type when searching
  - Filters work alongside directory and search query filters

- **📄 Context-Aware Terminology**
  - Preview modal shows "Page 1, Page 2..." for books
  - Preview modal shows "Slide 1, Slide 2..." for presentations
  - Search results display "5 pages" or "12 slides" accordingly
  - All UI labels automatically adapt to document type

### v0.4.2 - Native Desktop UI & Enhanced Controls
- **🎨 Professional Desktop Appearance**
  - Transformed UI to look like a native desktop application
  - Solid backgrounds, consistent orange buttons, subtle rounded corners
  - Removed web-responsive design (mobile/tablet breakpoints)
  - Fixed-width layout optimized for desktop screens
  
- **🛑 Scan Control & Feedback**
  - **Stop scan button** - Halt active scans with immediate visual feedback
  - **Processing timer** - See how long each file takes (with warnings for >10s)
  - **Enhanced progress display** - "Starting scan..." immediately, retains last file for 2s
  - **Clear cache confirmation** - Flowbite modal prevents accidental deletion
  
- **🎯 UI Improvements**
  - Help button moved to header (top-right)
  - Version badge prominently displayed next to title
  - Better scan status messages for cached vs. scanned files
  - Context-aware debug information display

### v0.4.1 - Cache Statistics Display
- **📊 Directory Cache Metrics**
  - Each directory now displays word count and cache size
  - See how much content is cached at a glance
  - Formatted with icons: 📝 for words, 💾 for storage size
  - Example: "1,495,503 words • 9.56 MB"
  
- **🐛 Directory Filtering Fixes**
  - Fixed search results for directories with spaces in names
  - Improved path matching logic for better reliability
  - Consistent filtering across all search operations

### v0.4.0 - Incremental Cache Saving
- **💾 Progressive Cache Persistence**
  - **Critical Fix**: Cache now saves after EACH file is indexed (not just at the end)
  - No more lost progress if scan is interrupted or app closes
  - Especially important for long OCR scans on scanned PDFs
  - Console shows "💾 Cache saved (items: X)" after each file
  - **Solves**: Calibre Library and other large collections not being cached
  
- **🔒 Crash-Resistant Indexing**
  - Already-indexed files are safe even if:
    - App closes mid-scan
    - System crashes
    - Scan manually stopped
    - OCR takes extremely long
  - Next scan will use cached results for completed files

### v0.3.1 - Live Scan Details & OCR Status
- **📊 Real-Time Scan Details Panel**
  - Beautiful visual panel shows detailed information for every file being scanned
  - Color-coded icons and status indicators for instant understanding
  - Shows checksum comparisons, modification times, and caching decisions
  - Combined with OCR status - see both scan reason and OCR progress
  
- **🔍 OCR Processing Visibility**
  - Purple-themed panel when OCR is running on scanned PDFs
  - Real-time status: "Extracting text from images..."
  - Visual feedback shows OCR progress and time expectations
  - Shows scan details alongside OCR status (not replacing it)
  
- **🗑️ Automatic Deleted File Cleanup**
  - Automatically removes deleted files from cache during scans
  - Shows removed count in scan summary
  - Keeps cache clean and relevant
  
- **⚡ Performance Optimizations**
  - Only calculates checksums when modification time changes
  - Quick cache check based on file mod time (instant)
  - Full checksum verification only for changed files
  - Dramatically faster rescans for unchanged collections

### v0.1.0 - Tauri-Only Release
- **🚀 Native Desktop App**: Converted to Tauri-only application
  - Removed Express server and Node.js backend code
  - Single native app with Rust backend for better performance
  - Smaller download size and faster startup
  - Full system integration with native file dialogs
  
- **✅ Checksum-Based Caching**: Prevents re-scanning unchanged files
  - SHA-256 checksums for reliable file change detection
  - **Scanned PDFs cached permanently** - OCR runs only once per file
  - Console logging shows cache status: `✓ Cached:` vs `⟳ Scanning:`
  - Dramatically faster rescans for large document collections
  - Checksums persist across app restarts

- **📊 Console Logging**: Real-time feedback during scanning
  - Run from terminal to see which files are cached vs. scanned
  - Clear visibility into OCR operations
  - Debug-friendly output for troubleshooting

### Search & Discovery
- **🎯 Search Highlighting**: All search terms are now highlighted in yellow throughout preview text
  - Supports single terms, quoted phrases, and wildcard patterns
  - Works in both slide text and snippet previews
  - Dark mode compatible with adjusted highlight colors
  
- **📁 Directory Filtering**: Checkbox controls for selective folder searching
  - Click checkboxes next to directories to include/exclude from search
  - "Select All/Deselect All" toggle for quick bulk selection
  - Visual badge shows number of active folders
  - Search results automatically filter based on directory selection
  
- **⚡ Manual Scanning**: Linking folders no longer triggers automatic indexing
  - Link folders instantly without waiting for scan to complete
  - Click "Rescan" button when ready to index files
  - Better control over when heavy indexing operations occur

### User Experience Enhancements
- **Clear Cache Button**: Easy cache management with one-click clearing functionality
- **Improved Scanning UI**: Cleaner progress display with file path shown below action buttons
- **Professional App Icon**: Custom-designed presentation icon with search functionality representation
- **Better Error Handling**: Enhanced error messages and troubleshooting guidance

### OCR & PDF Support
- **Native OCR Integration**: Calls external Tesseract OCR via Rust Command API
- **3-Tier Text Extraction**: Native PDF parsing → pdftotext → OCR (progressively tries each)
- **Dependency Detection**: Automatic detection and reporting of missing OCR dependencies
- **Visual Feedback**: Purple magnifying glass icon shows when OCR is running
- **Installation Guidance**: Clear instructions for installing required OCR tools
- **Smart Caching**: OCR results cached immediately (v0.4.0) - never lost on interruption

### Code Quality
- **Type Safety**: Enhanced TypeScript interfaces with checksum support
- **Error Handling**: Improved error handling in Rust backend with detailed console logging
- **Documentation**: Comprehensive inline documentation and help system
- **Incremental Saving**: Progressive cache persistence prevents data loss
- **Performance**: Two-tier caching strategy for 50-75x faster rescans

## 🛠️ Development

### Project Structure
```
src/                    # Frontend Svelte application
├── App.svelte         # Main application component
├── lib/
│   ├── api.ts         # Tauri IPC communication layer
│   └── components/    # Reusable UI components
src-tauri/             # Rust backend (Tauri)
├── src/
│   ├── main.rs        # Tauri application entry point
│   ├── scanner.rs     # File scanning and indexing logic
│   ├── state.rs       # Application state management
│   ├── models.rs      # Data structures
│   └── error.rs       # Error handling
├── Cargo.toml         # Rust dependencies
└── tauri.conf.json    # Tauri configuration
shared/                 # Shared utilities
├── types.ts           # TypeScript interfaces
└── search.ts          # Search functionality (used by frontend)
```

### Key Features Implementation

> **📖 For detailed technical documentation on the caching mechanism, see [CACHING-NOTES.md](./CACHING-NOTES.md)**

#### Checksum-Based Caching
- **SHA-256 Calculation**: Uses `sha2` crate in Rust for fast, reliable hashing with 8KB streaming buffer
- **Storage**: Checksums stored in `SlideIndexItem` struct and persisted to JSON
- **Incremental Saving (v0.4.0)**: Cache saves after EACH file is indexed (not just at end)
  - **Critical for OCR**: Slow OCR scans are now crash-resistant
  - **No lost progress**: Already-indexed files are safe if scan interrupted
  - **Console shows**: "💾 Cache saved (items: X)" after each file
  - **Solves**: Files requiring long OCR operations are now reliably cached
- **Two-Tier Caching Strategy (v0.3.0)**:
  1. **Quick check**: Compare modification time only (instant, no I/O)
  2. **Full check**: Calculate checksum only if mod time changed
  3. **Console output**: "✓ Cached (quick)" vs "✓ Cached (checksum)"
  4. **Performance**: 50-75x faster rescans for unchanged collections
- **Comparison Logic**:
  1. **Primary: Checksum comparison** - If both checksums exist and match → always cache (even if no content)
  2. **Fallback: Modification time** - If checksum unavailable, compare mod time
  3. **Rationale**: Files with no extractable text (scanned PDFs, encrypted PDFs) shouldn't be re-scanned if unchanged
- **Important Fixes**:
  - **v0.1.9**: Files with empty content are now correctly cached when checksums match
  - **v0.4.0**: Incremental saving prevents lost progress on interrupted scans

#### Clear Cache Functionality
- **Tauri Command**: `clear_cache` exposed to frontend via IPC
- **Implementation**: Clears all cached items and forces fresh scan on next index
- **UI**: Red "Clear Cache" button in action bar
- **Persistence**: State changes written to JSON file immediately

#### OCR Integration
- **External Tools**: Calls `pdftotext`, `pdftoppm`, and `tesseract` from Rust
- **Dependency Detection**: Automatic detection of missing OCR tools
- **Fallback Chain**: 
  1. Try native PDF text extraction
  2. Try `pdftotext` if available
  3. Try OCR with `pdftoppm` + `tesseract` if available
- **Error Reporting**: Clear guidance for installing missing dependencies

### Building & Testing
```bash
# Install dependencies
npm install

# Development mode
npm run tauri:dev    # Hot reload with Vite + Tauri

# Production build
npm run tauri:build  # Build app and DMG

# Frontend only
npm run build        # Build frontend assets
npm run dev          # Start Vite dev server (for UI testing)

# Type checking
npm run check        # Svelte + TypeScript validation
npm run lint         # Code linting (optional)
```

### Viewing Console Logs (Tauri App Only)

**This is a Tauri desktop application** - all Rust backend logs appear in Terminal, not in a browser console.

```bash
# Run installed app with console output
/Applications/Slides\ Indexer.app/Contents/MacOS/Slides\ Indexer

# Development mode (console output included automatically)
npm run tauri:dev

# Debug build (verbose logging)
./src-tauri/target/debug/bundle/macos/Slides\ Indexer.app/Contents/MacOS/Slides\ Indexer
```

**Console Output Includes:**
- `📊 Scan initialized: Existing cached items: 1082`
- `✓ Cached (quick): presentation.pptx` - Instant cache via mod time
- `⟳ Running OCR on PDF: scanned-doc.pdf` - OCR operations
- `💾 Cache saved (items: 1083)` - Incremental saves after each file
- `🗑️ Removed from cache (deleted): old-file.pdf` - Cleanup operations
- `━━━━ Scan Summary ━━━━` - Final statistics

**Troubleshooting:**
- Always run from Terminal to see what's happening
- Look for error messages, cache confirmations, and OCR status
- DevTools (`Cmd+Option+I` in debug builds) only needed for frontend UI issues

---

## 📋 Summary

**Slides Indexer v0.4.1** is a production-ready Tauri desktop application for macOS that solves the problem of searching through large collections of presentation files.

### Key Strengths

1. **Reliability**: Incremental cache saving ensures no lost progress
2. **Performance**: 50-75x faster rescans with two-tier caching
3. **OCR Support**: Automatically extracts text from scanned PDFs
4. **Visual Feedback**: Live scan details panel shows exactly what's happening
5. **Crash-Resistant**: Safe to interrupt scans - work is preserved

### Technical Highlights

- **Rust Backend**: High-performance file processing
- **SHA-256 Checksums**: Reliable file change detection
- **8KB Streaming**: Handles large files efficiently
- **3-Tier OCR**: Native parsing → pdftotext → Tesseract OCR
- **Automatic Cleanup**: Removes deleted files from cache

### Perfect For

- Lecturers managing course materials
- Researchers organizing academic papers
- Anyone with large presentation/PDF collections
- Users needing full-text search across documents
- Teams sharing presentation libraries

### Support & Documentation

- **[README.md](./README.md)** (this file) - Complete user guide and features
- **[QUICK-REFERENCE.md](./QUICK-REFERENCE.md)** - Fast lookup and common tasks
- **[CHANGELOG.md](./CHANGELOG.md)** - Version history and detailed changes
- **[CACHING-NOTES.md](./CACHING-NOTES.md)** - Technical deep-dive on caching mechanism
- **[TESTING-GUIDE.md](./TESTING-GUIDE.md)** - Testing procedures and verification
- **In-app Help** - Click Help button in app for quick reference

---

**Current Version**: 0.4.3  
**Platform**: macOS (Apple Silicon & Intel)  
**License**: See repository for details  
**Last Updated**: October 26, 2025
