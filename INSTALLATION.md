# Slides Indexer - Installation Guide

## 🎉 Installation Complete!

Your Slides Indexer application has been successfully installed with the new modern GUI and professional icon.

---

## 📍 Installation Location

**Application Path**: `/Applications/Slides Indexer.app`

The application is now available in your Applications folder and can be launched like any other macOS app.

---

## 🚀 How to Launch

### Method 1: Finder
1. Open **Finder**
2. Go to **Applications** folder
3. Double-click **Slides Indexer**

### Method 2: Spotlight
1. Press `Cmd + Space`
2. Type "Slides Indexer"
3. Press `Enter`

### Method 3: Launchpad
1. Open **Launchpad** (F4 or pinch gesture)
2. Search for "Slides Indexer"
3. Click the icon

### Method 4: Terminal
```bash
open "/Applications/Slides Indexer.app"
```

---

## 💾 Data Storage

**Cache Location**:
```
~/Library/Application Support/com.example.slidesindexer/slides-indexer/index.json
```

This file contains:
- All indexed documents
- SHA-256 checksums
- Cached OCR results
- Directory configurations

---

## ✨ New GUI Features

### Modern Design Elements
- **Glassmorphic Header**: Frosted glass effect with backdrop blur
- **Gradient Logo**: Orange-to-rose gradient icon with shadow
- **Enhanced Search**: Beautiful search input with focus animations
- **Colorful Cards**: Blue, Green, and Orange themed statistics
- **Smooth Animations**: 300ms transitions throughout
- **Professional Icon**: High-resolution icon for Dock and Finder

### Visual Improvements
- Gradient backgrounds
- Shadow effects with hover states
- Rounded corners (8px, 12px, 16px)
- Card lift animations
- Better typography and spacing

---

## 🎨 Icon Details

### Icon Format
- **Type**: macOS ICNS format
- **Resolutions**: 16x16 to 1024x1024 (Retina-ready)
- **Location**: `/Applications/Slides Indexer.app/Contents/Resources/icon.icns`

### Icon Features
- High-resolution for Retina displays
- Optimized for Dock and Finder
- Properly configured in Info.plist
- Cached and ready to display

---

## 🔧 Build Information

### Version
**v0.4.3** with Modern GUI

### Build Details
- **Bundle**: DMG and .app
- **Architecture**: Apple Silicon (aarch64)
- **Platform**: macOS
- **Framework**: Tauri + Rust + Svelte

### Build Artifacts
1. **Application**: `src-tauri/target/release/bundle/macos/Slides Indexer.app`
2. **DMG Installer**: `src-tauri/target/release/bundle/dmg/Slides Indexer_0.4.3_aarch64.dmg`

---

## 📋 Quick Start Guide

### 1. Link Directories
Click the **"Link folder"** button to add directories containing your presentations and PDFs.

### 2. Scan Documents
Click **"Rescan"** to index all PowerPoint (PPTX/PPT) and PDF files in linked directories.

### 3. Search
Use the search bar to find documents by:
- Keywords
- Exact phrases (in quotes)
- Wildcards (* or ?)

### 4. Filter Results
- **By Directory**: Use checkboxes to filter by folder
- **By Type**: Click "All", "Presentations", or "Books"

### 5. Preview & Open
- Click **"Preview"** to see slide contents
- Click **"Open"** to launch files in their native apps

---

## 🎯 Pro Tips

### OCR Support
For scanned PDFs, install OCR dependencies:
```bash
brew install poppler tesseract
```

### Cache Management
- **Clear Cache**: Use the "Clear Cache" button to force re-indexing
- **View Stats**: Each directory shows word count and cache size
- **OCR Results**: Cached automatically, never re-runs unless file changes

### Keyboard Shortcuts
- `Enter` in search bar: Execute search
- Click logo badge: Quick version check
- Right-click theme button: Reset to system theme

---

## 🔄 Updating

### To rebuild with changes:
```bash
cd /Volumes/STORAGE/SITES-2025/Slides
npm run tauri:build
```

### To reinstall:
```bash
rm -rf "/Applications/Slides Indexer.app"
cp -R "src-tauri/target/release/bundle/macos/Slides Indexer.app" /Applications/
touch "/Applications/Slides Indexer.app"
killall Dock
```

---

## 🐛 Troubleshooting

### Icon Not Showing
If the icon doesn't appear immediately:
```bash
touch "/Applications/Slides Indexer.app"
killall Dock
killall Finder
```

### App Won't Open (Security)
First launch may require security approval:
1. Go to **System Preferences** → **Security & Privacy**
2. Click **"Open Anyway"** if prompted
3. Or: Right-click app → **Open** → **Confirm**

### Reset Cache
To start fresh:
```bash
rm -rf ~/Library/Application\ Support/com.example.slidesindexer
```

### View Logs
Run from terminal to see debug output:
```bash
"/Applications/Slides Indexer.app/Contents/MacOS/Slides Indexer"
```

---

## 📊 Features Summary

### ✅ Search & Discovery
- Full-text search across all documents
- Keyword extraction and indexing
- Phrase matching and wildcards
- Real-time search results

### ✅ Smart Caching
- SHA-256 checksums for file tracking
- 50-75x faster rescans
- OCR results cached permanently
- Incremental cache saving

### ✅ Document Management
- Support for PPTX, PPT, and PDF
- Document type detection (Presentation/Book)
- Directory-based organization
- Cache statistics per folder

### ✅ User Experience
- Modern, beautiful interface
- Dark mode support
- Smooth animations
- Professional icon

---

## 🎨 About the New GUI

The modernized GUI features:
- **Gradients**: Subtle color transitions throughout
- **Glassmorphism**: Frosted glass effects
- **Shadows**: Layered shadows with colored glows
- **Animations**: Smooth transitions and hover effects
- **Typography**: Better hierarchy with bold headings
- **Colors**: Orange/Rose primary, Blue/Green/Slate secondary

All improvements maintain **100% backward compatibility** with existing functionality.

---

## 📝 Credits

**Application**: Slides Indexer v0.4.3
**GUI Design**: Modern redesign (November 2025)
**Framework**: Tauri + Rust + Svelte
**UI Library**: Tailwind CSS + Flowbite

---

## 🔗 Additional Resources

- **[GUI_IMPROVEMENTS.md](GUI_IMPROVEMENTS.md)** - Detailed list of UI enhancements
- **[BEFORE_AFTER_COMPARISON.md](BEFORE_AFTER_COMPARISON.md)** - Visual comparison guide
- **[README.md](README.md)** - Complete user guide
- **[CHANGELOG.md](CHANGELOG.md)** - Version history

---

**Enjoy your beautifully redesigned Slides Indexer!** 🎉
