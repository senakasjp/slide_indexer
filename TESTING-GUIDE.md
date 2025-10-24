# Testing Guide - Slides Indexer

**Version 0.4.0** | **Tauri Desktop Application**

This guide explains how to test the Slides Indexer application, particularly the caching and OCR features.

> **Important**: This is a Tauri application - all testing should be done with the **desktop app**, not in a browser. All debug output appears in **Terminal console**.

## 🧪 Testing Setup

### Prerequisites
```bash
# Install dependencies
npm install
brew install poppler tesseract

# Build debug version
npm run tauri build -- --debug
```

### Test Data Preparation

Create a test directory structure:
```
~/TestSlides/
├── presentations/
│   ├── lecture1.pptx
│   ├── lecture2.ppt
│   └── meeting-notes.pptx
├── pdfs/
│   ├── regular-text.pdf      # PDF with text layer
│   ├── scanned-document.pdf  # Scanned PDF requiring OCR
│   └── large-file.pdf        # 50MB+ PDF
```

## 🎯 Test Scenarios

### Test 1: Initial Scan & Cache Creation

**Objective:** Verify files are indexed and cached properly

**Steps:**
1. Run app from Terminal:
   ```bash
   ./src-tauri/target/debug/bundle/macos/Slides\ Indexer.app/Contents/MacOS/Slides\ Indexer
   ```
2. Link `~/TestSlides` folder
3. Click "Rescan"
4. Watch console output

**Expected Console Output:**
```
📊 Scan initialized:
  Existing cached items: 0
  Directories to scan: 1
    - /Users/you/TestSlides

⟳ Scanning: lecture1.pptx
💾 Cache saved (items: 1)

⟳ Scanning: lecture2.ppt
💾 Cache saved (items: 2)

⟳ Scanning: regular-text.pdf
💾 Cache saved (items: 3)

⟳ Running OCR on PDF: scanned-document.pdf
💾 Cache saved (items: 4)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Scan Summary:
  Total files:   4
  Scanned:       4 (newly processed)
  Cached:        0 (skipped, unchanged)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**Expected UI:**
- Scan details panel shows for each file
- OCR files show purple magnifying glass icon
- Final summary shows: "Total: 4, Scanned: 4, Cached: 0"

**Verify Cache:**
```bash
cat ~/Library/Application\ Support/com.example.slidesindexer/slides-indexer/index.json | python3 -c "import sys, json; print(f\"Items: {len(json.load(sys.stdin)['items'])}\")"
# Should output: Items: 4
```

### Test 2: Rescan Unchanged Files (Two-Tier Caching)

**Objective:** Verify quick caching via modification time

**Steps:**
1. Don't modify any files
2. Click "Rescan" again
3. Watch console

**Expected Console Output:**
```
📊 Scan initialized:
  Existing cached items: 4
  
✓ Cached (quick): lecture1.pptx
✓ Cached (quick): lecture2.ppt
✓ Cached (quick): regular-text.pdf
✓ Cached (quick): scanned-document.pdf

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Scan Summary:
  Total files:   4
  Scanned:       0 (newly processed)
  Cached:        4 (skipped, unchanged)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**Performance:**
- Should complete in <0.1 seconds
- No checksum calculations
- No file I/O except reading cache

**UI:**
- All files show green checkmark ✅
- Summary: "Scanned: 0, Cached: 4"

### Test 3: File Modification (Checksum Detection)

**Objective:** Verify checksum detects content changes

**Steps:**
1. Touch a file to change mod time: `touch ~/TestSlides/presentations/lecture1.pptx`
2. Click "Rescan"
3. Watch console

**Expected Console Output:**
```
✓ Cached (quick): lecture2.ppt
✓ Cached (quick): regular-text.pdf
✓ Cached (quick): scanned-document.pdf

⟳ Re-scanning (changed): lecture1.pptx - checksum changed: abc12345.. -> def67890..
💾 Cache saved (items: 4)

Scan Summary:
  Total files:   4
  Scanned:       1 (newly processed)
  Cached:        3 (skipped, unchanged)
```

**Verify:**
- Only the touched file is rescanned
- Others use quick cache
- New checksum is calculated and saved

### Test 4: Incremental Saving (v0.4.0)

**Objective:** Verify cache saves after each file, preventing data loss

**Steps:**
1. Link a folder with 10+ scanned PDFs
2. Start "Rescan"
3. **Close app after 3-4 files** are scanned (watch console)
4. Reopen app from Terminal
5. Click "Rescan" again

**Expected Behavior:**
- First scan console shows:
  ```
  ⟳ Running OCR on PDF: file1.pdf
  💾 Cache saved (items: 1083)
  ⟳ Running OCR on PDF: file2.pdf
  💾 Cache saved (items: 1084)
  [App closed here]
  ```
- Second scan console shows:
  ```
  ✓ Cached (quick): file1.pdf    # Saved before app closed
  ✓ Cached (quick): file2.pdf    # Saved before app closed
  ⟳ Running OCR on PDF: file3.pdf    # Continues from where it stopped
  💾 Cache saved (items: 1085)
  ```

**Success Criteria:**
- ✅ Files indexed before app close are cached
- ✅ OCR work is not repeated
- ✅ Scan continues from where it left off

### Test 5: Deleted File Cleanup

**Objective:** Verify deleted files are removed from cache

**Steps:**
1. Delete a file: `rm ~/TestSlides/presentations/lecture1.pptx`
2. Click "Rescan"
3. Watch console

**Expected Console Output:**
```
✓ Cached (quick): lecture2.ppt
✓ Cached (quick): regular-text.pdf

🗑️ Removed from cache (deleted): lecture1.pptx

Scan Summary:
  Total files:   3
  Scanned:       0
  Cached:        3
  Removed:       1 (deleted files)
```

**Verify:**
```bash
# File should not appear in cache
grep "lecture1.pptx" ~/Library/Application\ Support/com.example.slidesindexer/slides-indexer/index.json
# Should return nothing
```

### Test 6: OCR Operations

**Objective:** Verify OCR runs correctly and results are cached

**Steps:**
1. Add a scanned PDF (image-only, no text layer)
2. Run from Terminal to see OCR logs
3. Click "Rescan"

**Expected Console Output:**
```
⟳ Scanning: scanned-doc.pdf
⟳ Running OCR on PDF: scanned-doc.pdf
💾 Cache saved (items: X)

Scan Summary:
  Scanned: 1
```

**Expected UI:**
- Shows orange ✗ during initial scan
- Changes to purple 🔍 when OCR starts
- Scan details panel shows:
  ```
  ➕ New File Detected
  Checksum: Some("be53190d")
  
  ──────────────────────
  
  🔍 OCR Processing:
  🖼️  Extracting text from images...
  ⏳ This may take a few moments
  ```

**Second Scan:**
```
✓ Cached (quick): scanned-doc.pdf
```
- Should NOT run OCR again
- Should be instant

### Test 7: Large File Collections

**Objective:** Test performance with 100+ files

**Setup:**
- Link a folder with 100+ presentation files

**Measurements:**

**First Scan:**
```
Time: Depends on file types
- PPTX/PPT: ~100ms each
- PDFs with text: ~200ms each
- Scanned PDFs: ~5-30s each (OCR)

Console: 💾 Cache saved after EACH file
```

**Second Scan (No Changes):**
```
Time: <1 second for 100 files
Console: All show "✓ Cached (quick)"
Performance: ~0.001ms per file (mod time check)
```

**Third Scan (1 File Changed):**
```
Time: <1 second + time for 1 file
Console:
  - 99 files: "✓ Cached (quick)"
  - 1 file: "⟳ Re-scanning (changed)"
  - 1 file: "💾 Cache saved"
```

## 🐛 Common Test Failures & Solutions

### Files Not Caching

**Symptom:** Every scan shows "New File Detected" for same file

**Check:**
1. Look for `💾 Cache saved` in console
2. Check cache file exists and has content:
   ```bash
   cat ~/Library/Application\ Support/com.example.slidesindexer/slides-indexer/index.json
   ```
3. Verify no permission errors in console

**Solution:**
- Ensure app has full disk access (System Preferences → Security & Privacy)
- Check directory is linked in app settings
- Let scan complete fully - watch for cache save confirmations

### OCR Running Repeatedly

**Symptom:** OCR runs on every scan for same scanned PDF

**Root Cause (Pre-v0.4.0):**
- Cache only saved at end of scan
- Long OCR scans → app closed → cache never saved
- v0.4.0 fixes this with incremental saving

**Verify Fix:**
- Watch for `💾 Cache saved` after OCR completes
- Check file appears in cache JSON
- Next scan should show `✓ Cached (quick)`

### Performance Regression

**Symptom:** Rescans are slow even when no files changed

**Check Console:**
```
# Should see mostly:
✓ Cached (quick): ...

# Not this:
⟳ Re-scanning (changed): ...
```

**If many rescans:**
- Check modification times aren't changing
- Verify files aren't on network drive (unreliable timestamps)
- Look for checksum calculation failures

## 📊 Testing Checklist

Before releasing a new version, verify:

- [ ] Initial scan indexes all file types (PPTX, PPT, PDF)
- [ ] Cache saves after each file (`💾 Cache saved` in console)
- [ ] Second scan shows all files as `✓ Cached (quick)`
- [ ] Modified files detected via checksum
- [ ] Deleted files removed from cache
- [ ] OCR runs on scanned PDFs
- [ ] OCR results cached (no repeat OCR)
- [ ] App can be interrupted and resumed safely
- [ ] Scan details panel shows for all scanned files
- [ ] OCR status displays with purple icon
- [ ] Console logs are clear and informative
- [ ] Cache file persists at correct location

## 🔧 Debug Commands

```bash
# Check cache statistics
cat ~/Library/Application\ Support/com.example.slidesindexer/slides-indexer/index.json | python3 -c "import sys, json; data = json.load(sys.stdin); print(f'Items: {len(data[\"items\"])}, Dirs: {data[\"directories\"]}')"

# Find files from specific directory in cache
grep "Calibre Library" ~/Library/Application\ Support/com.example.slidesindexer/slides-indexer/index.json | wc -l

# Clear cache manually
rm ~/Library/Application\ Support/com.example.slidesindexer/slides-indexer/index.json

# Run app with full output
./src-tauri/target/debug/bundle/macos/Slides\ Indexer.app/Contents/MacOS/Slides\ Indexer 2>&1 | tee scan.log
```

## 📝 Reporting Issues

When reporting caching issues, include:

1. **Console output** - Copy from Terminal
2. **Scan details panel** - Screenshot from UI
3. **Cache stats** - Number of items, directories linked
4. **File type** - PPTX, PPT, or PDF? Scanned or text?
5. **Reproducibility** - Does it happen every time?

---

**See Also:**
- [README.md](./README.md) - User guide
- [QUICK-REFERENCE.md](./QUICK-REFERENCE.md) - Quick lookup
- [CACHING-NOTES.md](./CACHING-NOTES.md) - Technical details
- [CHANGELOG.md](./CHANGELOG.md) - Version history

