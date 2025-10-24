<script lang="ts">
  import { Card, Badge, Listgroup, ListgroupItem } from 'flowbite-svelte';
</script>

<section class="flex flex-col gap-6">
  <Card size="none" class="shadow-sm transition-colors dark:border-slate-800 dark:bg-slate-900/80">
    <header class="flex flex-wrap items-center justify-between gap-3">
      <div>
        <h2 class="text-2xl font-semibold text-slate-800 dark:text-slate-100">Slides Indexer Help</h2>
        <p class="text-sm text-slate-600 dark:text-slate-300">
          Native macOS desktop app for searching presentations and PDFs with intelligent caching.
        </p>
      </div>
      <Badge color="blue" class="text-xs uppercase tracking-wider dark:bg-primary-500/20 dark:text-primary-200">Version 0.4.1</Badge>
    </header>
  </Card>

  <Card size="none" class="shadow-sm transition-colors dark:border-slate-800 dark:bg-slate-900/80">
    <h3 class="text-lg font-semibold text-slate-800 dark:text-slate-100">🚀 Quick Start</h3>
    <Listgroup>
      <ListgroupItem class="text-sm leading-relaxed text-slate-600 dark:bg-transparent dark:text-slate-300">
        <span class="fa-solid fa-folder-plus me-2 text-blue-500"></span>
        <strong>Link Folders:</strong> Click "Link folder" to add directories containing your presentations and PDFs.
      </ListgroupItem>
      <ListgroupItem class="text-sm leading-relaxed text-slate-600 dark:bg-transparent dark:text-slate-300">
        <span class="fa-solid fa-rotate me-2 text-green-500"></span>
        <strong>Scan Files:</strong> Click "Rescan" to index all PowerPoint (.pptx, .ppt) and PDF files.
        Watch the progress indicator for live scan details.
      </ListgroupItem>
      <ListgroupItem class="text-sm leading-relaxed text-slate-600 dark:bg-transparent dark:text-slate-300">
        <span class="fa-solid fa-magnifying-glass me-2 text-purple-500"></span>
        <strong>Search:</strong> Type your query and press <kbd class="rounded bg-slate-200 px-1.5 py-0.5 text-xs dark:bg-slate-700">Enter</kbd> 
        or click the search button. Results appear instantly.
      </ListgroupItem>
      <ListgroupItem class="text-sm leading-relaxed text-slate-600 dark:bg-transparent dark:text-slate-300">
        <span class="fa-solid fa-eye me-2 text-orange-500"></span>
        <strong>Preview:</strong> Click any result to see slide previews with search terms highlighted in yellow.
      </ListgroupItem>
    </Listgroup>
  </Card>

  <Card size="none" class="shadow-sm transition-colors dark:border-slate-800 dark:bg-slate-900/80">
    <h3 class="text-lg font-semibold text-slate-800 dark:text-slate-100">🔍 Advanced Search</h3>
    <ul class="space-y-3 text-sm leading-relaxed text-slate-600 dark:text-slate-300">
      <li class="flex items-start gap-2">
        <span class="fa-solid fa-quote-left text-blue-500"></span>
        <div>
          <strong>Exact Phrases:</strong> Use double quotes for exact phrase matching<br>
          <code class="font-mono text-xs">"machine learning"</code> → Finds exact phrase<br>
          <code class="font-mono text-xs">"network security"</code> → Words must appear together
        </div>
      </li>
      <li class="flex items-start gap-2">
        <span class="fa-solid fa-asterisk text-purple-500"></span>
        <div>
          <strong>Wildcards:</strong> Use * for multiple characters, ? for single<br>
          <code class="font-mono text-xs">data*</code> → Matches: data, database, dataset<br>
          <code class="font-mono text-xs">comput?r</code> → Matches: computer, computar
        </div>
      </li>
      <li class="flex items-start gap-2">
        <span class="fa-solid fa-layer-group text-green-500"></span>
        <div>
          <strong>Combined Queries:</strong> Mix phrases, wildcards, and terms<br>
          <code class="font-mono text-xs">"deep learning" python* neural</code><br>
          → Exact phrase + wildcard + regular term
        </div>
      </li>
      <li class="flex items-start gap-2">
        <span class="fa-solid fa-keyboard text-amber-500"></span>
        <div>
          <strong>Search Trigger:</strong> Type your query and press <kbd class="rounded bg-slate-200 px-1.5 py-0.5 text-xs dark:bg-slate-700">Enter</kbd> 
          or click the search button. Search does NOT run automatically as you type.
        </div>
      </li>
      <li class="flex items-start gap-2">
        <span class="fa-solid fa-highlighter text-yellow-500"></span>
        <span>
          <strong>Search Highlighting:</strong> All search terms are highlighted in yellow throughout
          preview text for easy identification.
        </span>
      </li>
      <li class="flex items-start gap-2">
        <span class="fa-solid fa-filter text-indigo-500"></span>
        <span>
          <strong>Directory Filtering:</strong> Use checkboxes next to directories to include/exclude
          folders from search results.
        </span>
      </li>
    </ul>
  </Card>

  <Card size="none" class="shadow-sm transition-colors dark:border-slate-800 dark:bg-slate-900/80">
    <h3 class="text-lg font-semibold text-slate-800 dark:text-slate-100">📊 Scanning & Caching</h3>
    <ul class="space-y-3 text-sm leading-relaxed text-slate-600 dark:text-slate-300">
      <li class="flex items-start gap-2">
        <span class="fa-solid fa-bolt text-green-500"></span>
        <div>
          <strong>Smart Caching:</strong> Files are only re-scanned when actually changed<br>
          <span class="text-xs">✓ Green check = cached (instant), ✗ Orange cross = scanning, 🔍 Purple = OCR</span>
        </div>
      </li>
      <li class="flex items-start gap-2">
        <span class="fa-solid fa-fingerprint text-purple-500"></span>
        <div>
          <strong>Checksums:</strong> SHA-256 checksums detect file changes reliably<br>
          <span class="text-xs">First scan: calculates checksums | Next scans: ultra-fast (50-75x faster)</span>
        </div>
      </li>
      <li class="flex items-start gap-2">
        <span class="fa-solid fa-floppy-disk text-blue-500"></span>
        <div>
          <strong>Incremental Saving:</strong> Cache saves after EACH file (v0.4.0+)<br>
          <span class="text-xs">Safe to interrupt - already-indexed files are preserved</span>
        </div>
      </li>
      <li class="flex items-start gap-2">
        <span class="fa-solid fa-magnifying-glass text-purple-500"></span>
        <div>
          <strong>OCR for Scanned PDFs:</strong> Automatically extracts text from image-based PDFs<br>
          <span class="text-xs">Requires: <code class="font-mono">brew install poppler tesseract</code></span><br>
          <span class="text-xs">OCR runs once per file, then cached - never repeated</span>
        </div>
      </li>
      <li class="flex items-start gap-2">
        <span class="fa-solid fa-trash-can text-red-500"></span>
        <div>
          <strong>Automatic Cleanup:</strong> Deleted files removed from cache automatically<br>
          <span class="text-xs">Scan summary shows: "Removed: X (deleted files)"</span>
        </div>
      </li>
    </ul>
  </Card>

  <Card size="none" class="shadow-sm transition-colors dark:border-slate-800 dark:bg-slate-900/80">
    <h3 class="text-lg font-semibold text-slate-800 dark:text-slate-100">📺 Live Scan Details</h3>
    <p class="text-sm leading-relaxed text-slate-600 dark:text-slate-300 mb-3">
      During scanning, a visual panel shows detailed information for each file:
    </p>
    <ul class="space-y-3 text-sm leading-relaxed text-slate-600 dark:text-slate-300">
      <li class="flex items-start gap-2">
        <span class="fa-solid fa-info-circle text-blue-500"></span>
        <div>
          <strong>Blue Panel (Normal Scan):</strong> Shows why file is being scanned<br>
          <span class="text-xs">Displays: checksums, modification times, file status (new/changed)</span>
        </div>
      </li>
      <li class="flex items-start gap-2">
        <span class="fa-solid fa-magnifying-glass text-purple-500"></span>
        <div>
          <strong>Purple Panel (OCR):</strong> Shows when extracting text from images<br>
          <span class="text-xs">Message: "Extracting text from images... This may take a few moments"</span>
        </div>
      </li>
      <li class="flex items-start gap-2">
        <span class="fa-solid fa-check-circle text-green-500"></span>
        <div>
          <strong>Scan Summary:</strong> After completion shows total/scanned/cached/removed counts<br>
          <span class="text-xs">Example: Total: 1082, Scanned: 12, Cached: 1067, Removed: 3</span>
        </div>
      </li>
    </ul>
  </Card>

  <Card size="none" class="shadow-sm transition-colors dark:border-slate-800 dark:bg-slate-900/80">
    <h3 class="text-lg font-semibold text-slate-800 dark:text-slate-100">🐛 Troubleshooting</h3>
    <ul class="space-y-3 text-sm leading-relaxed text-slate-600 dark:text-slate-300">
      <li class="flex items-start gap-2">
        <span class="fa-solid fa-terminal text-slate-500"></span>
        <div>
          <strong>Viewing Console Logs:</strong> This is a Tauri desktop app - run from Terminal to see logs<br>
          <code class="font-mono text-xs">/Applications/Slides\ Indexer.app/Contents/MacOS/Slides\ Indexer</code><br>
          <span class="text-xs">All debugging info appears in Terminal, not browser console</span>
        </div>
      </li>
      <li class="flex items-start gap-2">
        <span class="fa-solid fa-repeat text-orange-500"></span>
        <div>
          <strong>Files Always Rescanning:</strong> Check scan details panel for reason<br>
          <span class="text-xs">Look for: modification time changes, checksum mismatches, or "New File Detected"</span>
        </div>
      </li>
      <li class="flex items-start gap-2">
        <span class="fa-solid fa-file-pdf text-red-500"></span>
        <div>
          <strong>OCR Not Working:</strong> Install required dependencies<br>
          <code class="font-mono text-xs">brew install poppler tesseract</code> (macOS)<br>
          <span class="text-xs">App will show warning if dependencies are missing</span>
        </div>
      </li>
      <li class="flex items-start gap-2">
        <span class="fa-solid fa-floppy-disk text-blue-500"></span>
        <div>
          <strong>Cache Not Saving:</strong> Watch Terminal for "💾 Cache saved (items: X)"<br>
          <span class="text-xs">Location: ~/Library/Application Support/com.example.slidesindexer/slides-indexer/index.json</span>
        </div>
      </li>
      <li class="flex items-start gap-2">
        <span class="fa-solid fa-trash text-red-500"></span>
        <div>
          <strong>Reset Everything:</strong> Click "Clear Cache" button<br>
          <span class="text-xs">Forces complete re-index on next scan</span>
        </div>
      </li>
      <li class="flex items-start gap-2">
        <span class="fa-solid fa-lock text-amber-500"></span>
        <span>
          <strong>Permissions:</strong> Grant app "Full Disk Access" in System Preferences → 
          Security & Privacy if folders don't appear in picker.
        </span>
      </li>
    </ul>
  </Card>

  <Card size="none" class="shadow-sm transition-colors dark:border-slate-800 dark:bg-slate-900/80">
    <h3 class="text-lg font-semibold text-slate-800 dark:text-slate-100">💡 Performance Tips</h3>
    <ul class="space-y-2 text-sm leading-relaxed text-slate-600 dark:text-slate-300">
      <li class="flex items-start gap-2">
        <span class="fa-solid fa-gauge-high text-green-500"></span>
        <span>
          <strong>First scan</strong> may take time (especially for scanned PDFs with OCR). 
          Subsequent scans are near-instant for unchanged files.
        </span>
      </li>
      <li class="flex items-start gap-2">
        <span class="fa-solid fa-circle-check text-blue-500"></span>
        <span>
          <strong>Safe to interrupt:</strong> Cache saves after each file. Close app mid-scan, 
          reopen, and continue - already-scanned files are cached.
        </span>
      </li>
      <li class="flex items-start gap-2">
        <span class="fa-solid fa-rocket text-purple-500"></span>
        <span>
          <strong>Expected performance:</strong> 500 unchanged files rescan in ~0.2 seconds 
          (50-75x faster than v0.2.x).
        </span>
      </li>
    </ul>
  </Card>

  <Card size="none" class="shadow-sm transition-colors dark:border-slate-800 dark:bg-slate-900/80">
    <h3 class="text-lg font-semibold text-slate-800 dark:text-slate-100">📖 Documentation</h3>
    <p class="text-sm text-slate-600 dark:text-slate-300 mb-3">
      For detailed technical documentation, see the project repository:
    </p>
    <ul class="space-y-2 text-sm text-slate-600 dark:text-slate-300">
      <li class="flex items-start gap-2">
        <span class="fa-solid fa-book text-blue-500"></span>
        <span><strong>README.md</strong> - Complete user guide and features</span>
      </li>
      <li class="flex items-start gap-2">
        <span class="fa-solid fa-bolt text-purple-500"></span>
        <span><strong>QUICK-REFERENCE.md</strong> - Fast lookup and common tasks</span>
      </li>
      <li class="flex items-start gap-2">
        <span class="fa-solid fa-timeline text-green-500"></span>
        <span><strong>CHANGELOG.md</strong> - Version history and updates</span>
      </li>
      <li class="flex items-start gap-2">
        <span class="fa-solid fa-code text-orange-500"></span>
        <span><strong>CACHING-NOTES.md</strong> - Technical deep-dive on caching</span>
      </li>
      <li class="flex items-start gap-2">
        <span class="fa-solid fa-vial text-indigo-500"></span>
        <span><strong>TESTING-GUIDE.md</strong> - Testing procedures</span>
      </li>
    </ul>
  </Card>
</section>
