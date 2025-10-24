# Slides Indexer - Quick Reference Guide

**Version 0.4.0** | **Tauri Desktop Application** | **macOS Only**

## 🚀 Getting Started

### Installation
```bash
# Install from DMG
Slides Indexer_0.4.0_aarch64.dmg

# Run with console logs
/Applications/Slides\ Indexer.app/Contents/MacOS/Slides\ Indexer
```

### First Time Setup
1. Launch the app
2. Click "Link folder" → Select your presentation directories
3. Click "Rescan" → Wait for initial indexing (may take time for OCR)
4. Watch console for progress: `💾 Cache saved (items: X)`

## 🔍 Common Tasks

### Search Presentations
- Type in search box (real-time results)
- Use quotes for exact phrases: `"machine learning"`
- Use wildcards: `data*` or `comput?r`
- Click checkboxes to filter by directory

### Rescan Files
```
Rescan → Scans ALL linked folders
[📁 folder icon] → Rescans ONE specific folder
```

### View Console Logs
```bash
# Always run from Terminal for debugging
/Applications/Slides\ Indexer.app/Contents/MacOS/Slides\ Indexer
```

### Clear Cache
Click "Clear Cache" button → Forces complete re-index on next scan

## 📊 Understanding Scan Status

### Visual Indicators (In App)
- ✅ **Green checkmark** - File cached (skipped)
- 🔍 **Purple magnifying glass** - OCR running (extracting text from images)
- ✗ **Orange cross** - File being scanned/re-scanned

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

## 🐛 Troubleshooting

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

---

**Version**: 0.4.0  
**Platform**: macOS (Apple Silicon & Intel)  
**App Type**: Tauri Desktop Application  
**Console Logs**: Required for debugging (run from Terminal)

