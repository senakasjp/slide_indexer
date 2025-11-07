# Slides Indexer - Comprehensive Architecture Documentation

**Application Version**: 0.4.3  
**Platform**: macOS (Tauri Native Desktop App)  
**Built With**: Rust (Backend) + Svelte 4 (Frontend)

---

## 1. APPLICATION ARCHITECTURE OVERVIEW

### High-Level Design: Tauri Desktop Application

This is a **native desktop application** (not a web app), built with Tauri - a framework that combines Rust backend with a Svelte TypeScript frontend in a single native executable.

```
┌─────────────────────────────────────────────────────────────┐
│                     Slides Indexer App                      │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────────────┐      ┌──────────────────────┐   │
│  │   Svelte Frontend    │      │   Rust Backend       │   │
│  │  (TypeScript)        │◄────►│   (Tauri)            │   │
│  │                      │      │                      │   │
│  │ • UI Components      │      │ • File Scanning      │   │
│  │ • State Management   │      │ • Text Extraction    │   │
│  │ • Search/Filter      │      │ • OCR Integration    │   │
│  │ • Preview Display    │      │ • Caching System     │   │
│  │                      │      │ • State Persistence  │   │
│  └──────────────────────┘      └──────────────────────┘   │
│           │                              │                 │
│           └──────────────────────────────┘                 │
│          Tauri IPC (Type-Safe Commands)                    │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐  │
│  │        Operating System (File System Access)         │  │
│  │  • Native file dialogs                               │  │
│  │  • File reading/writing                              │  │
│  │  • External command execution (OCR tools)            │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### Communication Pattern: IPC Commands

Frontend calls Rust backend through **Tauri's type-safe IPC system**:

```typescript
// Frontend (src/lib/api.ts)
const invoke = await waitForTauriInvoke();
return invoke<ScanSummary>('rescan');  // Calls Rust command synchronously

// Backend (src-tauri/src/main.rs)
#[tauri::command]
async fn rescan(manager: State<Arc<StateManager>>) -> CommandResult<ScanSummary> {
    // Executes Rust code asynchronously
}
```

### Event System: Backend → Frontend

Backend emits events through Tauri's event system for real-time updates:

```rust
// In Rust (state.rs)
self.app_handle.emit_all("scan-progress", payload);

// In Svelte (App.svelte)
const { listen } = await import('@tauri-apps/api/event');
await listen('scan-progress', (event) => {
    // Update UI with real-time scan progress
});
```

---

## 2. KEY COMPONENTS

### Frontend Architecture (Svelte)

#### Main Application Component: `src/App.svelte` (2066 lines)

**Responsibilities**:
- Central state management and orchestration
- User interface rendering with Flowbite components
- Search query handling and result filtering
- Directory and document type filtering
- Preview modal display
- Theme management (light/dark mode)
- Error handling and user feedback

**Key State Variables**:
```typescript
let directories: string[] = [];           // Linked directories
let items: SlideIndexItem[] = [];         // All indexed items
let filteredItems: SlideIndexItem[] = []; // Search results
let query: string = '';                   // Current search query
let selectedDirectories: Set<string>;     // Directory filter selection
let documentTypeFilter: 'all' | 'presentation' | 'book' = 'all';
let isDarkMode: boolean = false;
let isRescanning: boolean = false;        // Scan in progress
let currentScanPath: string | null = null; // Current file being scanned
let currentDebugInfo: string | null = null; // Scan details display
```

**Key Features Implemented**:
- Real-time search with debouncing
- Directory selection/deselection with bulk toggles
- Document type filtering (new in v0.4.3)
- Preview modal with slide navigation
- Scan progress display with processing time
- Theme persistence and system preference detection
- Offline fallback mode with localStorage

**Search Flow** (in `App.svelte`):
```
User types query
    ↓
Debounce 300ms
    ↓
Parse search query
    ↓
Filter by selected directories
    ↓
Filter by document type
    ↓
Apply search matching (name, path, snippet, keywords, slides)
    ↓
Display results with highlighting
```

#### API Layer: `src/lib/api.ts` (180 lines)

**Purpose**: Abstract Tauri IPC invocation layer

**Key Functions**:
```typescript
fetchState()                    // Get current app state
updateDirectories(dirs)         // Add/remove directories (no scan)
rescan()                        // Index all directories
rescanDirectory(dir)            // Index specific directory
searchIndex(query)              // Full-text search
openSlideDeck(id)               // Open file in native app
clearCache()                    // Clear all cached items
selectDirectories()             // Native directory picker
```

**Smart Detection**: Automatically detects Tauri environment vs. web environment, with graceful fallback.

#### UI Components: `src/lib/components/`

- **SearchInput.svelte**: Real-time search input with clear button
- **HelpContent.svelte**: In-app help and usage guide

#### Shared Utilities: `shared/` Directory

**`shared/types.ts`**: TypeScript interfaces for all data structures
```typescript
interface SlideIndexItem {
  id: string;                  // SHA-1 hash of path
  path: string;                // Full file path
  name: string;                // Filename
  kind: 'pptx' | 'ppt' | 'pdf';
  slideCount: number | null;   // Number of slides/pages
  snippet: string;             // First 240 chars of content
  keywords: string[];          // Extracted keywords (max 40)
  updatedAt: number;           // File modification time (ms)
  slides: SlidePreview[];       // Preview of each slide
  checksum?: string;           // SHA-256 hash
  documentType?: 'presentation' | 'book';
}
```

**`shared/search.ts`**: Search query parsing and matching
- Supports exact phrases (quoted text)
- Wildcard patterns (* and ?)
- Multiple search terms (AND logic)
- Case-insensitive matching

---

## 3. RUST BACKEND: `src-tauri/src/`

### Core Modules

#### `main.rs` (164 lines)

**Entry Point and Command Handlers**:
- Tauri app initialization
- Command handler registration (IPC endpoints)
- DevTools setup in debug mode
- State management initialization

**Exposed Tauri Commands**:
```rust
#[tauri::command]
fn fetch_state(manager) -> CommandResult<AppState>
#[tauri::command]
async fn update_directories(manager, directories) -> CommandResult<ScanSummary>
#[tauri::command]
async fn rescan(manager) -> CommandResult<ScanSummary>
#[tauri::command]
async fn rescan_directory(manager, directory) -> CommandResult<ScanSummary>
#[tauri::command]
fn search_index(manager, query) -> CommandResult<SearchResponse>
#[tauri::command]
fn open_slide_deck(_app, manager, id) -> CommandResult<()>
#[tauri::command]
fn clear_cache(manager) -> CommandResult<()>
```

All async operations use `async_runtime::spawn_blocking()` for thread-safe execution.

#### `state.rs` (335 lines)

**Purpose**: Application state management and persistence

**StateManager Struct**:
```rust
pub struct StateManager {
    state: Mutex<AppState>,           // Thread-safe state
    storage_path: PathBuf,            // ~/.../slides-indexer/index.json
    app_handle: AppHandle,            // Tauri handle for events
}
```

**Responsibilities**:
1. **State Persistence**: Loads/saves to JSON file on startup and after each operation
2. **Scan Orchestration**: Manages scanning with progress callbacks
3. **Incremental Cache Saving**: Saves state after EACH file indexed (v0.4.0+)
4. **Directory Management**: Add/remove/scan directories
5. **Search**: Query processing and filtering
6. **Event Emission**: Sends scan-progress events to frontend

**Key Implementation**:
```rust
// Incremental saving during scan
let mut on_item_indexed = |item: SlideIndexItem| {
    // Update state with new item
    state.items.push(item);
    state.last_indexed_at = Some(current_timestamp());
    // SAVE IMMEDIATELY after each file
    persist_state(&storage_path, &state)?;
    println!("💾 Cache saved (items: {})", state.items.len());
};

// Pass callback to scanner
scan_directories(&directories, &existing, &mut progress_cb, &mut on_item_indexed);
```

This ensures no work is lost if the scan is interrupted.

#### `scanner.rs` (1609 lines)

**The Heart of the Application**: File scanning, text extraction, OCR integration, and caching logic.

**Main Functions**:

**`scan_directories()` - The Scan Engine**
```rust
pub fn scan_directories(
    directories: &[String],
    existing: &[SlideIndexItem],
    progress: &mut dyn FnMut(&str, &str, Option<&str>),
    on_item_indexed: &mut dyn FnMut(SlideIndexItem),
) -> Result<ScanOutcome>
```

Flow:
1. Build map of existing cached items
2. Recursively find all PPTX, PPT, and PDF files
3. For each file:
   - Check if should cache (two-tier strategy)
   - If needs indexing: extract text, derive keywords, create previews
   - Call `on_item_indexed()` immediately (incremental save)
4. Cleanup deleted files from cache
5. Return scan summary

**Two-Tier Caching Strategy** (v0.3.0+):

```
┌─ For each file ──────────────────────────┐
│                                          │
│  Tier 1: Quick Check (Instant)          │
│  ┌──────────────────────────────────┐   │
│  │ Compare mod_time with existing   │   │
│  │ If unchanged → CACHE & skip      │   │
│  │ Time: ~0.001ms                   │   │
│  └──────────────────────────────────┘   │
│           ↓ (if changed)                │
│  Tier 2: Checksum Verification         │
│  ┌──────────────────────────────────┐   │
│  │ Calculate SHA-256 (8KB buffer)   │   │
│  │ Compare with existing checksum   │   │
│  │ If match → CACHE (content same)  │   │
│  │ If mismatch → RESCAN             │   │
│  │ Time: ~50-200ms for large files  │   │
│  └──────────────────────────────────┘   │
│                                          │
│  Performance: 99% cached (Tier 1)       │
│              <1% need checksum          │
│              Rarely need full rescan    │
└──────────────────────────────────────────┘
```

**File Processing Functions**:

1. **`index_pptx()` - PowerPoint Processing**
   - Open PPTX as ZIP archive
   - Extract XML from `ppt/slides/slide*.xml`
   - Regex extract text runs: `<a:t[^>]*>(.*?)</a:t>`
   - Clean and filter noise tokens
   - Extract keywords from combined text
   - Determine document type: Always "Presentation"
   - Calculated mod time and checksum
   - Result: `SlideIndexItem` with slide previews

2. **`index_ppt()` - Legacy PowerPoint**
   - Read binary file as ASCII
   - Convert non-printable chars to spaces
   - Extract XML/text content
   - Single preview if content exists
   - Simpler than PPTX (binary format)

3. **`index_pdf()` - PDF Processing** (Complex 3-tier approach)

   **Tier 1: Native PDF Stream Parsing**
   - Find PDF stream segments
   - Decompress FlateDecode streams
   - Extract text using regex patterns
   - Detect page count and orientation
   
   **Tier 2: pdftotext Fallback**
   - If native parsing yields no text
   - Run external `pdftotext` command
   - Parse output into pages
   
   **Tier 3: OCR as Last Resort**
   - If still no meaningful text
   - Convert PDF to PNG images (`pdftoppm`)
   - Run OCR on each page (`tesseract`)
   - Combine extracted text
   
   **Document Type Detection**:
   ```rust
   // Detect orientation from PDF MediaBox
   let is_landscape = detect_pdf_orientation(&content);
   let doc_type = if is_landscape {
       DocumentType::Presentation
   } else {
       DocumentType::Book
   };
   ```

**Text Processing Pipeline**:
```
Raw Content
    ↓
strip_xml_tags()         // Remove XML markup
    ↓
strip_binary_artifacts() // Convert non-ASCII
    ↓
filter_noise_tokens()    // Remove noise words (slide, title, etc.)
    ↓
cleanup_whitespace()     // Normalize spaces/newlines
    ↓
[Meaningful text check]
    ↓
derive_keywords()        // Extract top N keywords
truncate_snippet()       // First 240 chars
build_previews()         // Create per-slide/page previews
```

**Keyword Extraction** (`derive_keywords()`):
- Tokenize text with regex: `[a-z0-9]{3,}`
- Count frequency of each token
- Filter out tokens that appear in slide content (avoid noise)
- Sort by frequency and take top 40 keywords

**Checksum Calculation** (`calculate_file_checksum()`):
```rust
fn calculate_file_checksum(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];  // 8KB streaming buffer
    
    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 { break; }
        hasher.update(&buffer[..bytes_read]);
    }
    
    Ok(hex::encode(hasher.finalize()))
}
```
- Uses streaming (not loading entire file into memory)
- Handles large files efficiently (50MB+)
- SHA-256 provides reliable file change detection

**Search Pattern Matching** (`matches_query()`):
```rust
pub fn matches_query(item: &SlideIndexItem, pattern: &SearchPattern) -> bool {
    if pattern.is_empty { return true; }
    
    let corpus = build_search_corpus(item);
    // Corpus includes: name, path, snippet, slides, keywords
    
    // All phrases must match (AND logic)
    for phrase in &pattern.phrases {
        if !corpus.contains(phrase) { return false; }
    }
    
    // All terms must match (AND logic)
    for term in &pattern.terms {
        if !corpus.contains(term) { return false; }
    }
    
    // All wildcards must match (AND logic)
    for wildcard in &pattern.wildcards {
        if !wildcard.is_match(&corpus) { return false; }
    }
    
    true
}
```

**OCR Integration**:
```
PDF with no text
    ↓
detect: No text extracted
    ↓
Run: pdftoppm → convert PDF to PNG images
     ↓ (if available)
Run: tesseract → OCR images to text
     ↓ (if available)
Result: Text extracted for searching
    ↓
Cache: OCR result stored, never re-runs
       (unless file content changes)
```

**External Command Resolution**:
```rust
fn resolve_command(command: &str) -> Option<PathBuf> {
    // Search in standard locations:
    // macOS: /opt/homebrew/bin, /usr/local/bin, /usr/bin
    // Linux: /usr/local/bin, /usr/bin, /bin, /snap/bin
    // Windows: Program Files directories
}
```

#### `models.rs` (94 lines)

**Data Structure Definitions**:

```rust
pub struct SlideIndexItem {
    pub id: String,                          // SHA-1 of path
    pub path: String,
    pub name: String,
    pub kind: SlideKind,                     // PPTX, PPT, PDF
    pub slide_count: Option<u32>,
    pub snippet: String,                     // Preview (240 chars)
    pub keywords: Vec<String>,               // Top 40 keywords
    pub updated_at: u64,                     // Mod time in ms
    pub slides: Vec<SlidePreview>,           // Per-slide content
    pub checksum: Option<String>,            // SHA-256 hash
    pub document_type: Option<DocumentType>, // Presentation|Book
}

pub struct AppState {
    pub directories: Vec<String>,            // Linked directories
    pub items: Vec<SlideIndexItem>,          // All indexed items
    pub last_indexed_at: Option<u64>,        // Last scan timestamp
    pub warnings: Vec<String>,               // OCR/error warnings
}

pub enum SlideKind { Pptx, Pdf, Ppt }
pub enum DocumentType { Presentation, Book }
```

#### `error.rs` (44 lines)

**Error Handling**:
```rust
pub enum AppError {
    Message(String),
    IoError(std::io::Error),
    JsonError(serde_json::Error),
    // ... other error types
}
```

Implements `thiserror` for ergonomic error conversion.

---

## 4. FILE PROCESSING DETAILED FLOW

### PowerPoint (PPTX) Processing

```
PPTX File (ZIP Archive)
    ↓
Open as ZIP
    ↓
Find entries: ppt/slides/slide*.xml
    ↓
For each XML:
  Extract text runs with regex: <a:t[^>]*>(.*?)</a:t>
  Strip XML tags
  Remove binary artifacts
  Filter noise tokens
  Clean whitespace
  ↓
Create SlidePreview for each slide
Combine all text for keywords
Derive top 40 keywords
Truncate to 240 char snippet
    ↓
SlideIndexItem {
  kind: Pptx,
  slide_count: 15,
  snippet: "First 240 chars...",
  keywords: ["machine", "learning", ...],
  slides: [{ index: 1, text: "..." }, ...],
  document_type: Presentation,  // Always
  checksum: "abc123...",
}
```

### PDF Processing

```
PDF File (Binary)
    ↓
Read full file into buffer
    ↓
Parse PDF Structure:
  Find stream segments
  Decompress FlateDecode
  Extract text with regex
  Count pages
  Detect orientation (width > height = landscape)
    ↓
Text Extraction (3-Tier):
  ├─ Tier 1: Native PDF parsing
  │   └─ If meaningful text: Done!
  │
  ├─ Tier 2: pdftotext (if installed)
  │   └─ If meaningful text: Done!
  │
  └─ Tier 3: OCR (if installed & text still missing)
      ├─ Convert PDF → PNG images
      ├─ Run tesseract on each PNG
      └─ Combine OCR results
    ↓
Document Type Detection:
  Landscape (width > height) → Presentation
  Portrait (height > width) → Book
    ↓
Build Previews:
  Create SlidePreview for each page
  Extract keywords from combined text
    ↓
SlideIndexItem {
  kind: Pdf,
  slide_count: 42,
  snippet: "Extracted text...",
  keywords: [...],
  slides: [{ index: 1, text: "..." }, ...],
  document_type: Presentation,  // or Book
  checksum: "def456...",
}
```

---

## 5. CACHING SYSTEM: Checksum-Based Intelligent Caching

### Purpose
Avoid re-scanning unchanged files, especially critical for PDFs with OCR (which can take 5-30 seconds per file).

### How It Works

**Scenario 1: File Unchanged (Modification Time Unchanged)**
```
Second Scan:
  File.updated_at == Cached.updated_at
  → ✓ CACHE (quick, instant)
  → Skip checksum calculation
  → Reuse all cached data
  Console: "✓ Cached (quick): document.pdf"
  Performance: ~0.001ms per file
```

**Scenario 2: File Changed (Modification Time Changed)**
```
Second Scan:
  File.updated_at != Cached.updated_at
  → Calculate SHA-256 checksum
  → Compare with cached checksum
  
  If checksums MATCH:
    Content unchanged (e.g., metadata-only change)
    → ✓ CACHE (checksum verified)
    → Skip full text extraction
    → Reuse OCR results
    Console: "✓ Cached (checksum): document.pdf"
    Performance: ~50-200ms per file
  
  If checksums DON'T MATCH:
    Content actually changed
    → ⟳ RESCAN
    → Re-extract text
    → Re-run OCR if needed
    Console: "⟳ Re-scanning (changed): document.pdf"
    Performance: 5-30 seconds for OCR PDFs
```

**Scenario 3: Scanned PDF (No Text)**
```
First Scan:
  PDF has no extractable text (scanned image)
  Try native parsing → No text
  Try pdftotext → No text
  Try OCR → No text extracted
  
  Cache Result:
  {
    snippet: "",
    keywords: [],
    checksum: "130ade7d..."
  }

Second Scan:
  File mod_time unchanged
  → ✓ CACHE (quick)
  → Never runs OCR again on this file
  
  Even if mod_time changed:
  Calculate checksum: "130ade7d..."
  Checksums match → ✓ CACHE (checksum)
  → No OCR needed
```

**Key Design Decision (v0.1.9)**:
Always cache when checksums match, **even if content is empty**. This is crucial for scanned PDFs and encrypted files that have no extractable text.

### Incremental Cache Saving (v0.4.0+)

**Before v0.4.0**:
- Scan all files first
- Save cache at END of scan
- If interrupted: All progress lost

**After v0.4.0**:
```rust
let mut on_item_indexed = |item: SlideIndexItem| {
    state.items.push(item);
    // Save immediately
    persist_state(&storage_path, &state)?;
    println!("💾 Cache saved (items: {})", state.items.len());
};

// Scanner calls this after EACH file
scan_directories(..., &mut on_item_indexed);
```

**Benefits**:
- OCR work never lost
- Safe to interrupt scans
- Crash-resistant
- Shows progress with cache confirmations

### Storage Format

```json
{
  "directories": ["/Users/name/Documents/Presentations"],
  "items": [
    {
      "id": "abc123def...",
      "path": "/Users/name/Documents/Presentations/lecture1.pdf",
      "name": "lecture1.pdf",
      "kind": "pdf",
      "slideCount": 45,
      "snippet": "Machine learning fundamentals...",
      "keywords": ["machine", "learning", "classification", ...],
      "updatedAt": 1698500000000,
      "slides": [
        { "index": 1, "text": "Title slide content..." },
        { "index": 2, "text": "Introduction to ML..." }
      ],
      "checksum": "sha256hexstring1234...",
      "documentType": "presentation"
    }
  ],
  "lastIndexedAt": 1698502000000,
  "warnings": []
}
```

**Location**: `~/Library/Application Support/com.example.slidesindexer/slides-indexer/index.json`

---

## 6. SEARCH FUNCTIONALITY

### Search Query Types

**1. Exact Phrases** (Quoted):
```
Query: "machine learning"
Result: Items containing "machine learning" together
```

**2. Keyword Terms** (Unquoted):
```
Query: machine learning algorithm
Result: Items containing ALL of (machine, learning, algorithm)
        Logic: AND
```

**3. Wildcards**:
```
Query: mach*
Result: Items containing words starting with "mach"
        (machine, machinimation, etc.)

Query: m?chine
Result: Items containing "m[any char]chine"
```

**4. Combined**:
```
Query: "machine learning" algorithm p*ocessing
Result: Items matching ALL of:
  - Exact phrase "machine learning"
  - Term "algorithm"
  - Wildcard pattern "p*ocessing"
```

### Search Corpus Construction

For each item, search against:
```
[name] [path] [snippet] [slide_texts...] [keywords...]
```

All converted to lowercase for case-insensitive matching.

### Search Flow (Frontend)

```
User types in search input
    ↓
Debounce 300ms (wait for typing to stop)
    ↓
Trim whitespace
    ↓
IF empty query:
  Show all items filtered by document type
ELSE:
  send searchIndex(query) to backend
    ↓
Backend processes in state.rs:
  Parse query with SearchPattern::new()
  Filter items with matches_query()
  Return matching items
    ↓
Frontend displays results with:
  - Item name, snippet, file type
  - Document type badge (Presentation/Book)
  - Slide/page count
  - File icon
  - Keywords
    ↓
User can:
  - Click item to preview
  - Filter by document type
  - Filter by directory
  - See highlighted matches in preview
```

### Document Type Filtering

```
Filter Buttons: [All] [Presentations] [Books]

Filtering Logic:
  'all' → Show all items
  'presentation' → Show items where documentType == 'presentation'
  'book' → Show items where documentType == 'book'

Combined with Directory Filtering:
  Selected directories AND document type filter
  Both conditions must be satisfied
```

### Search Highlighting (Frontend Only)

Display search matches highlighted in yellow in preview text.

---

## 7. DATA FLOW: File System → Indexing → Search → Display

### Complete End-to-End Flow

```
┌──────────────────────────────────────────────────────────┐
│ USER WORKFLOW                                             │
└──────────────────────────────────────────────────────────┘

1. LINK FOLDERS
   User clicks "Link folder" → Native file dialog → Selects directories
   ↓
   updateDirectories([paths]) → Backend saves to state
   ↓
   No automatic scan (v0.4.2+)

2. RESCAN
   User clicks "Rescan" button
   ↓
   rescan() → Backend starts scanning all directories
   ↓

┌──────────────────────────────────────────────────────────┐
│ SCANNING PROCESS (Backend)                                │
└──────────────────────────────────────────────────────────┘

   For each directory:
     Find all *.pptx, *.ppt, *.pdf files
     ↓
     For each file:
       ├─ Check cache (2-tier strategy)
       │  ├─ Tier 1: Mod time match → Use cached
       │  └─ Tier 2: Checksum match → Use cached
       │
       └─ If not cached:
          ├─ Index file (extract text)
          ├─ Call on_item_indexed() callback
          └─ Save state to disk immediately (v0.4.0+)
   ↓
   Cleanup: Remove deleted files from cache
   ↓
   Return ScanSummary { indexed, scanned, cached, errors }

┌──────────────────────────────────────────────────────────┐
│ PROGRESS FEEDBACK (Real-Time Events)                      │
└──────────────────────────────────────────────────────────┘

   Backend emits scan-progress events:
   {
     path: "/path/to/file.pdf",
     status: "scanning" | "cached" | "ocr",
     debugInfo: "Rescan Information: ..."
   }
   ↓
   Frontend receives and updates UI:
   - Current file being processed
   - Processing time counter
   - Scan details panel
   - Progress indicator

┌──────────────────────────────────────────────────────────┐
│ SEARCHING (Backend + Frontend)                            │
└──────────────────────────────────────────────────────────┘

   User enters search query
   ↓
   Frontend debounces (300ms)
   ↓
   searchIndex(query) → Backend
   ↓
   Backend: SearchPattern::new(query)
   - Parse terms, phrases, wildcards
   ↓
   Backend: Filter items
   - For each item: matches_query()
   - Build search corpus (name, path, snippet, slides, keywords)
   - Check all terms/phrases/wildcards match
   ↓
   Backend: Return SearchResponse {
     items: [matching SlideIndexItems],
     total: count,
     lastIndexedAt: timestamp
   }
   ↓
   Frontend: Apply additional filters
   - Directory filter (selected directories)
   - Document type filter (Presentation/Book)
   ↓
   Frontend: Display filtered results
   - Sorted by relevance
   - Show document type badge
   - Show snippet preview
   - Show keywords

┌──────────────────────────────────────────────────────────┐
│ PREVIEW & OPENING                                         │
└──────────────────────────────────────────────────────────┘

   User clicks item in results
   ↓
   Frontend: Show modal with:
   - All slides/pages with text
   - Navigation controls
   - File info (type, path, date)
   ↓
   User clicks "Open" button
   ↓
   openSlideDeck(itemId) → Backend
   ↓
   Backend: Find item by id
   ↓
   Launch file with native app:
   - macOS: open /path/to/file
   - Windows: start "" "path\to\file"
   - Linux: xdg-open /path/to/file
```

---

## 8. STATE MANAGEMENT

### Initial Load (`onMount`):

```typescript
const state = await fetchState();
directories = state.directories;
items = state.items;
selectedDirectories = new Set(state.directories);
lastIndexedAt = state.lastIndexedAt;
warnings = state.warnings;
```

### State Persistence:

**Frontend (Svelte)**:
- Uses localStorage for theme preference
- LocalForage fallback for offline mode
- No caching of indexed items (all from backend)

**Backend (Rust)**:
- Saves to JSON after:
  - Adding/removing directories
  - Each file indexed (incremental)
  - Clearing cache
- Loads on startup
- Thread-safe with Mutex

### State Structure:

```rust
AppState {
  directories: Vec<String>,           // Linked paths
  items: Vec<SlideIndexItem>,         // All indexed items
  last_indexed_at: Option<u64>,       // Last scan timestamp
  warnings: Vec<String>,              // OCR/error messages
}
```

---

## 9. TECHNOLOGY STACK DETAILS

### Frontend Dependencies

```json
{
  "svelte": "^4.2.12",                    // Component framework
  "@tauri-apps/api": "^1.5.3",            // Tauri IPC
  "flowbite-svelte": "^0.44.24",          // UI components
  "@fortawesome/fontawesome-free": "^6.5.1", // Icons
  "jszip": "^3.10.1",                     // ZIP reading (future use)
  "localforage": "^1.10.0",               // Offline storage
  "tailwindcss": "^3.3.7"                 // Styling
}
```

### Backend Dependencies (Cargo.toml)

```toml
tauri = { version = "1.5", features = ["dialog-open", "fs-all", "shell-open"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"                      # JSON serialization
sha2 = "0.10"                           # SHA-256 checksums
sha1 = "0.10"                           # SHA-1 for item IDs
hex = "0.4"                             # Hex encoding
zip = { version = "0.6", features = ["deflate"] }  # PPTX ZIP reading
flate2 = { version = "1.0" }            # Zlib decompression
regex = "1.10"                          # Text pattern matching
globwalk = "0.8"                        # Recursive file finding
once_cell = "1.19"                      # Lazy static initialization
thiserror = "1.0"                       # Error handling
anyhow = "1.0"                          # Result type
tempfile = "3.10"                       # Temp file handling (OCR)
```

### Build Tools

- **Vite**: Frontend bundler with hot reload
- **Rust Cargo**: Backend compilation
- **Tauri CLI**: Desktop app bundling
- **Biome**: Linting and formatting

---

## 10. PERFORMANCE CHARACTERISTICS

### Scanning Performance

| Operation | Time | Factor |
|-----------|------|--------|
| Cache hit (mod time) | ~0.001ms | Instant per file |
| Checksum calculation | ~50-200ms | For time-changed files |
| Text extraction (PPTX) | ~100-500ms | Depends on slide count |
| Text extraction (PDF native) | ~50-200ms | Stream parsing |
| PDF text (pdftotext) | ~200-500ms | External tool |
| PDF text (OCR) | 5-30s | Tesseract per PDF |

### Caching Impact

**Without caching** (scanning 1000 items):
- 100+ PDFs with OCR = 500 seconds+ (8+ minutes)

**With caching** (subsequent scans if unchanged):
- 99% cached via mod time = <1 second
- 1% via checksum = ~50-200ms each
- Final rescan: <10 seconds for 1000 items

**Two-tier strategy benefit**: ~50-75x faster rescans

### Memory Usage

- Streaming checksums: Constant 8KB buffer
- ZIP reading: File streamed, not fully loaded
- PDF parsing: Full file in memory (problematic for 100MB+ PDFs)
- State: All indexed items in memory (JSON serialization)

---

## 11. ERROR HANDLING

### OCR Dependency Detection

```rust
// At startup, detect available tools
fn resolve_command(command: &str) -> Option<PathBuf> {
    // Searches PATH and default locations
    // macOS: /opt/homebrew/bin, /usr/local/bin
    // Linux: /usr/local/bin, /usr/bin, /bin
    // Windows: Program Files directories
}

// If missing:
pub fn ocr_status_message() -> Option<String> {
    Some("PDF extraction tools missing: pdftoppm, tesseract. 
          Install them to enable full PDF scanning.")
}
```

### Graceful Degradation

```rust
// Try native parsing
if let Ok(native_text) = extract_pdf_contents() {
    if has_meaningful_text() { return; }
}

// Try pdftotext
if let Ok(pdftotext_text) = extract_pdf_with_pdftotext() {
    if has_meaningful_text() { return; }
}

// Try OCR
if let Ok(ocr_text) = extract_pdf_with_ocr() {
    if has_meaningful_text() { return; }
}

// If all fail, cache empty content
cache.snippet = "";
cache.keywords = [];
```

### Error Reporting

Errors collected during scan and reported in UI:
```typescript
scanErrors: string[] = [];
```

Examples:
- "Failed to index PDF /path: File too large"
- "PDF extraction tools missing: tesseract..."

---

## 12. PLATFORM SUPPORT & DIFFERENCES

### macOS (Primary)

- Uses `open` command to launch files
- Homebrew for OCR tools (`brew install poppler tesseract`)
- App bundle: `~/Applications/Slides Indexer.app`
- Data: `~/Library/Application Support/com.example.slidesindexer/`

### Windows (Supported in Code)

- Uses `start` command (hidden window)
- Program Files for OCR tools
- MSI installer or portable

### Linux (Supported in Code)

- Uses `xdg-open` command
- System package manager for OCR tools

---

## 13. KEY ARCHITECTURAL DECISIONS

### Why Tauri?

1. **Single executable** - Easier distribution than Electron
2. **Smaller bundle** - ~50MB vs. 150MB for Electron
3. **Native performance** - Rust backend for file I/O
4. **System integration** - Native file dialogs, OS commands

### Why Two-Tier Caching?

1. **99% hit rate on mod time** - Instant skipping
2. **Checksum fallback** - Handles modified-time issues
3. **Perfect for OCR** - Never re-runs expensive OCR unless content changes
4. **Incremental saving** - No lost progress on interruption

### Why Search on Backend?

1. **Large datasets** - Faster pattern matching in Rust
2. **Type-safe** - Serde serialization
3. **Centralized state** - Single source of truth
4. **Future extensibility** - Could add full-text indexing

### Why JSON Storage?

1. **Simplicity** - Easy to inspect/debug
2. **Human-readable** - Can manually edit if needed
3. **Standard format** - Portable
4. **No database dependency** - Self-contained

---

## 14. FUTURE EXTENSIBILITY

### Potential Improvements

1. **Full-text indexing** - Currently linear search, could use Tantivy
2. **Database** - Replace JSON with SQLite for 100k+ items
3. **Partial OCR** - Only OCR first N pages of scanned PDFs
4. **Parallel scanning** - Currently sequential
5. **Incremental scanning** - Watch file system for changes
6. **Export results** - Save search results to CSV/JSON
7. **Bookmarks** - Mark favorite documents
8. **Tags** - User-applied metadata

---

## Summary

**Slides Indexer** is a sophisticated desktop application that combines:

1. **High-performance file scanning** with intelligent checksum-based caching
2. **Multi-format support** (PPTX, PPT, PDF) with fallback mechanisms
3. **OCR integration** for scanned documents with crash-resistant incremental saving
4. **Smart search** with phrase, wildcard, and term matching
5. **Document classification** distinguishing presentations from books
6. **Real-time feedback** during long operations
7. **Offline capability** with graceful degradation
8. **Cross-platform foundation** (macOS primary, Windows/Linux supported)

The architecture separates concerns effectively:
- **Frontend**: Responsive UI, filtering, display
- **Backend**: File processing, state management, search
- **IPC**: Type-safe communication between layers

Performance is optimized through multi-tier caching and streaming file processing, enabling fast rescans of large collections.

