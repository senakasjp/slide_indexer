# Slides Indexer - Quick Reference Guide

**Version 0.4.3** | **Tauri Desktop Application** | **macOS Only**

## 🚀 Getting Started

### Installation
```bash
# Install from DMG
Slides Indexer_0.4.3_aarch64.dmg

# Run with console logs
/Applications/Slides\ Indexer.app/Contents/MacOS/Slides\ Indexer
```

### First Time Setup
1. Launch the app
2. Click "Link folder" → Select your presentation directories
3. Click "Rescan" → Wait for initial indexing (may take time for OCR)
4. Watch console for progress: `💾 Cache saved (items: X)`

## 🔍 Common Tasks

### Search and Filter
- Type in search box (real-time results)
- Use quotes for exact phrases: `"machine learning"`
- Use wildcards: `data*` or `comput?r`
- Click checkboxes to filter by directory
- **Filter by type**: Click "Presentations" or "Books" button without searching to see all of that type

### Rescan Files
```
Rescan → Scans ALL linked folders
[📁 folder icon] → Rescans ONE specific folder
Stop scan → Halts active scan (replaces Rescan button during scan)
```

### View Console Logs
```bash
# Always run from Terminal for debugging
/Applications/Slides\ Indexer.app/Contents/MacOS/Slides\ Indexer
```

### Clear Cache
Click "Clear Cache" button → Confirmation modal appears → Confirm to clear cache

## 📊 Understanding Scan Status

### Version Badge (v0.4.2+)
- Displayed in header next to app title
- Current version: `0.4.3`
- Small badge with wrench/tool icon
- Helpful for verifying installed version

### Document Type Badges (v0.4.3)
Each document in search results shows its type:
- **📊 Blue "Presentation" badge** - Slides and decks (Beamer, PowerPoint)
- **📚 Green "Book" badge** - Books and documents (Calibre Library, portrait PDFs)
- Badge displayed next to file type (PPTX, PDF, PPT)
- Automatic detection based on PDF page orientation

### Smart Terminology (v0.4.3)
UI automatically adapts based on document type:
- **Books**: Show "5 pages", preview displays "Page 1", "Page 2", etc.
- **Presentations**: Show "12 slides", preview displays "Slide 1", "Slide 2", etc.
- Count labels update automatically in search results
- Preview modal headers change based on content type

### Document Type Filter (v0.4.3)
Three filter buttons above search results:
- **"All"** - Show all documents (default)
- **"Presentations"** - Show only landscape PDFs and PowerPoint files
- **"Books"** - Show only portrait PDFs
- **No search required** - Click any button to instantly see all documents of that type
- Active filter button highlighted in blue/green
- Works with directory filters and search queries

### Cache Statistics (v0.4.1+)
Each directory in the list shows:
- **📝 Word count** - Total words cached (e.g., "1,495,503 words")
- **💾 Cache size** - Storage used (e.g., "9.56 MB")
- Updates automatically after rescans

### Visual Indicators (In App)
- ✅ **Green checkmark** - File cached (skipped)
- 🔍 **Purple magnifying glass** - OCR running (extracting text from images)
- ✗ **Orange cross** - File being scanned/re-scanned

### Processing Time (v0.4.2)
During active scans, the UI shows:
- "Processing for X seconds..." - Real-time timer for current file
- Warning indicator for files taking longer than 10 seconds
- Helps identify slow OCR operations or large files
- Timer resets automatically when switching to next file

### Scan Progress Display (v0.4.2)
- **Starting scan...** - Appears immediately when scan begins
- **File path with status icon** - Shows current file being processed
- **Status messages**:
  - "File retrieved from cache (no scan needed)" - Cached files (green ✓)
  - "Processing file..." - Actively scanned files (orange ✗)
- Progress display persists for 2 seconds after scan completes

### Console Messages
```
✓ Cached (quick): file.pptx          # Instant cache (mod time match)
✓ Cached (checksum): file.pdf        # Cache via checksum (time changed, content same)
⟳ Scanning: new-file.pptx            # New file or content changed
⟳ Running OCR on PDF: scanned.pdf    # OCR operation in progress
💾 Cache saved (items: 1083)         # Cache persisted to disk
🗑️ Removed from cache: deleted.pdf   # Deleted file cleanup
```

### Scan Details Panel

Shows for every file being scanned:

**Blue Panel (Normal Scan):**
```
ℹ️ SCAN DETAILS
➕ New File Detected
First time indexing this file
Checksum: Some("be53190d")
```

**Purple Panel (OCR):**
```
🔍 OCR PROCESSING
🔍 OCR Processing:
🖼️  Extracting text from images...
⏳ This may take a few moments
```

## 🎨 UI Features (v0.4.2)

### Native Desktop Appearance
- **Solid backgrounds** - No translucent effects
- **Consistent orange buttons** - All primary actions use same color
- **Subtle rounded corners** - Professional, not overly rounded
- **Fixed-width layout** - Optimized for desktop (no responsive design)
- **Clean header** - Title, version badge, and Help button only

### Action Buttons
- **Link folder** - Orange button, adds new directories
- **Rescan** - Orange button, indexes all linked folders
- **Stop scan** - Red button (replaces Rescan during active scan)
- **Clear Cache** - Dark red button, shows confirmation modal
- **Help** - Located in header (top-right corner)

### Confirmation Modals
- **Clear Cache** - Warning modal before clearing
  - Explains that action is permanent
  - "Cancel" button to abort
  - "Clear Cache" button to confirm (red)

## 🐛 Troubleshooting

### Scan Progress Not Showing
**Symptoms:**
- "Starting scan..." appears but no file paths
- Files are being scanned (console shows activity) but UI is blank

**Causes & Solutions:**
- **Normal for cached files**: Green checkmark (✓) files don't show detailed progress
- **Clear cache first**: Click "Clear Cache" → Confirm → Rescan to see full debug logs
- **Check version**: Look for version badge in header (should show `0.4.2`)

### Stop Scan Not Working
**Expected behavior:**
- Click "Stop scan" → Button shows "Stopping..." with spinner
- Scan progress clears immediately
- Backend may continue briefly (can't instantly kill OCR processes)
- UI stops updating with new files

**If not working:**
- Check version badge (should be `0.4.2`)
- Run from Terminal to see console logs
- Look for "Scan stopped by user" message

### File Always Shows "New File Detected"

**Fixed in v0.4.0** with incremental saving!

**Check:**
```bash
# Verify file is in cache
grep "your-filename.pdf" ~/Library/Application\ Support/com.example.slidesindexer/slides-indexer/index.json

# Watch console for cache saves
# Should see: 💾 Cache saved (items: X) after each file
```

**Solution:**
- Let scan complete fully
- Watch for `💾 Cache saved` confirmations in console
- Next scan should show "✓ Cached"

### OCR Running Every Time

**Cause:** File not in cache (see above)

**Verify OCR is working:**
```bash
# Check if tools are installed
which pdftoppm tesseract pdftotext

# Install if missing
brew install poppler tesseract
```

### Scanned PDFs Taking Forever

**Expected:**
- First OCR scan: 5-30 seconds per PDF page (up to 40 pages max)
- Subsequent scans: Instant (cached)

**v0.4.0 Improvement:**
- OCR work saved immediately after each file
- Safe to close app - already-scanned PDFs are cached

### Cache Not Persisting

**Check cache location:**
```bash
cat ~/Library/Application\ Support/com.example.slidesindexer/slides-indexer/index.json
```

**Look for in console:**
- `💾 Cache saved (items: X)` - Should appear after each scanned file
- `⚠️ Failed to save cache` - Indicates disk write error

## ⚡ Performance Tips

### Optimal Workflow
1. **Link folders** - Add all directories at once
2. **First scan** - Let it run completely (may take time for OCR)
3. **Watch console** - Confirm `💾 Cache saved` appears
4. **Subsequent scans** - Should be near-instant with `✓ Cached (quick)`

### Expected Performance
- **Unchanged files**: <0.1 seconds for 500 files (mod time check)
- **Changed files**: ~50-200ms each (checksum calculation)
- **OCR files**: 5-30 seconds first time, instant thereafter

### Cache Efficiency
```
Scan Summary:
  Total files:   1082
  Scanned:       12  (new or changed)
  Cached:        1070 (skipped, unchanged)
  Removed:       3  (deleted files)
```

## 🔧 Development

### Build Commands
```bash
# Development
npm run tauri:dev

# Debug build (verbose logs + DevTools)
npm run tauri build -- --debug

# Release build (optimized)
npm run tauri build
```

### Key Files
- `src-tauri/src/scanner.rs` - Caching logic, OCR integration
- `src-tauri/src/state.rs` - Incremental saving, state management
- `src/App.svelte` - UI and scan details panel
- `shared/types.ts` - TypeScript interfaces

### Testing Strategy
1. Use **debug builds** for development (verbose console output)
2. Always run from **Terminal** to see logs
3. Check **scan details panel** in UI for visual feedback
4. Verify **cache saves** in console after each file
5. Test **interruption recovery** - close app mid-scan, reopen, rescan

## 📚 Documentation

- **[README.md](./README.md)** - Complete user guide and features
- **[QUICK-REFERENCE.md](./QUICK-REFERENCE.md)** (this file) - Fast lookup guide
- **[CHANGELOG.md](./CHANGELOG.md)** - Detailed version history
- **[CACHING-NOTES.md](./CACHING-NOTES.md)** - Technical deep-dive on caching
- **[TESTING-GUIDE.md](./TESTING-GUIDE.md)** - Testing procedures and verification

## 🎯 Best Practices

### When to Clear Cache
- Search results seem outdated or incorrect
- Want to see full scan progress with debug logs
- Troubleshooting caching issues
- After major app update

### When to Use Stop Scan
- Accidentally started scan on wrong directory
- Need to make changes before scan completes
- Testing/debugging specific files
- Long OCR scan needs to be interrupted

### Verifying Installation
1. Check version badge in header → Should show `0.4.3`
2. Click "Clear Cache" → Modal should appear (not immediate action)
3. Start a rescan → "Stop scan" button should appear (red)
4. Look for orange buttons (Link folder, Rescan, Search)
5. Check UI has solid backgrounds (not translucent)
6. Look for document type filter buttons above results
7. Check for "Presentation" or "Book" badges in search results

### How Document Type Detection Works
**For PDFs:**
- App extracts MediaBox dimensions from PDF structure
- Compares width vs height of first page
- **Landscape (width > height)** = Presentation (Beamer slides)
- **Portrait (height > width)** = Book (Calibre Library, documents)

**For PowerPoint (PPTX/PPT):**
- Always classified as "Presentation"
- No page orientation detection needed

**Use Cases:**
- Separate Beamer lecture slides from textbook PDFs
- Browse all presentations without searching
- Browse all books from Calibre Library
- Find only presentations in mixed document library
- Filter out books when searching for slides
- Quickly identify document type at a glance

**Benefits:**
- Clear distinction between reference books and presentation materials
- No search needed to browse by document type
- Appropriate terminology (pages vs slides) for each document type
- Easier navigation with context-aware labels

---

**Version**: 0.4.3  
**Platform**: macOS (Apple Silicon & Intel)  
**App Type**: Tauri Desktop Application  
**Console Logs**: Required for debugging (run from Terminal)

