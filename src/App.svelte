<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { Alert, Badge, Button, Card, Input, Label, Modal, Spinner } from 'flowbite-svelte';
import localforage from 'localforage';
import JSZip from 'jszip';
import type { AppState, SlideIndexItem, SlidePreview } from '../shared/types';
import {
  fetchState,
  openSlideDeck,
  rescan as rescanAllApi,
  rescanDirectory as rescanDirectoryApi,
  searchIndex,
  selectDirectories,
  updateDirectories,
  clearCache as clearCacheApi
} from './lib/api';
import { matchesSearchQuery, parseSearchQuery } from '../shared/search';
import { extractPdfPagesWeb } from '../shared/pdf-web';
  import HelpContent from './lib/components/HelpContent.svelte';
import SearchInput from './lib/components/SearchInput.svelte';

  type FileWithPath = File & { webkitRelativePath?: string };
  type View = 'index' | 'help';

const STORE_KEY = 'slides-indexer:state';
const THEME_STORAGE_KEY = 'slides-indexer:theme';
  const MAX_SNIPPET_LENGTH = 240;
  const MAX_KEYWORDS = 40;
  const latin1Decoder = new TextDecoder('latin1');

  const isGibberish = (input: string): boolean => {
    const compact = input.replace(/\s+/g, '');
    if (compact.length < 40) {
      return false;
    }
    const alphaMatches = compact.match(/[a-zA-Z]/g) ?? [];
    if (!alphaMatches.length) {
      return true;
    }
    const alphaRatio = alphaMatches.length / compact.length;
    if (alphaRatio < 0.35) {
      return true;
    }
    const upperMatches = compact.match(/[A-Z]/g) ?? [];
    if (upperMatches.length / alphaMatches.length > 0.9 && alphaMatches.length > 80) {
      return true;
    }
    const longTokens = input.split(/\s+/).filter((token) => token.length > 40).length;
    if (longTokens > 2) {
      return true;
    }
    const normalizedTokens = input
      .split(/\s+/)
      .map((token) => token.replace(/[^a-zA-Z0-9]/g, '').toLowerCase())
      .filter(Boolean);
    const rawTokens = input.split(/\s+/).filter(Boolean);
    if (rawTokens.length >= 20) {
      const singleCharRatio = rawTokens.filter((token) => token.length === 1).length / rawTokens.length;
      if (singleCharRatio > 0.4) {
        return true;
      }
      const vowelRatio = rawTokens.filter((token) => /[aeiou]/i.test(token)).length / rawTokens.length;
      if (vowelRatio < 0.2) {
        return true;
      }
      if (normalizedTokens.length) {
        const uniqueTokens = new Set(normalizedTokens);
        if (uniqueTokens.size <= Math.max(4, Math.ceil(normalizedTokens.length * 0.1))) {
          return true;
        }
      }
    }
    return false;
  };

  const hasMeaningfulText = (input: string): boolean => {
    const trimmed = input.trim();
    if (!trimmed) {
      return false;
    }
    if (trimmed.length < 12) {
      return /[a-zA-Z0-9]/.test(trimmed);
    }
    const alphaMatches = trimmed.match(/[a-zA-Z]/g) ?? [];
    if (!alphaMatches.length) {
      return false;
    }
    const tokens = trimmed.split(/\s+/).filter(Boolean);
    if (!tokens.length) {
      return false;
    }
    if (tokens.length >= 6) {
      const multiCharRatio = tokens.filter((token) => token.length > 1).length / tokens.length;
      if (multiCharRatio < 0.4) {
        return false;
      }
      const vowelRatio = tokens.filter((token) => /[aeiou]/i.test(token)).length / tokens.length;
      if (vowelRatio < 0.2) {
        return false;
      }
    }
    if (isGibberish(trimmed)) {
      return false;
    }
    return true;
  };


  const APP_VERSION = '0.4.3';

  const DEFAULT_STATE: AppState = {
    directories: [],
    items: [],
    lastIndexedAt: null,
    warnings: []
  };

  let folderInput: HTMLInputElement | null = null;
  const HEADER_SEARCH_ID = 'search-header';
  let directories: string[] = [];
  let items: SlideIndexItem[] = [];
  let filteredItems: SlideIndexItem[] = [];
  let hasActiveQuery = false;
  let query = '';
  let isLoading = true;
  let isSearching = false;
  let isRescanning = false;
  let rescanningDirectory: string | null = null;
  let scanStopped = false;
let showDirectoryModal = false;
let showClearCacheModal = false;
let newDirectory = '';
let lastIndexedAt: number | null = null;
let loadError: string | null = null;
let offlineNotice: string | null = null;
let scanErrors: string[] = [];
let searchError: string | null = null;
let isOfflineMode = false;
let view: View = 'index';
let isDarkMode = false;
let searchDebounce: ReturnType<typeof setTimeout> | null = null;
let searchToken = 0;
let removeHashListener: (() => void) | null = null;
let removeThemeListener: (() => void) | null = null;
let removeScanListener: (() => void) | null = null;
let currentScanPath: string | null = null;
let currentScanStatus: string | null = null;
let currentDebugInfo: string | null = null;
let currentFileStartTime: number | null = null;
let processingTimeInterval: ReturnType<typeof setInterval> | null = null;
let processingTime = 0;
let documentTypeFilter: 'all' | 'presentation' | 'book' = 'all';
let systemMediaQuery: MediaQueryList | null = null;
let lastScanSummary: { total: number; scanned: number; cached: number } | null = null;
let gridClass = 'grid gap-4';
let pdfExtractionWarning = false;
let showPreviewModal = false;
let selectedItem: SlideIndexItem | null = null;
let selectedSlides: SlidePreview[] = [];
let isOpeningDeck = false;
let openDeckError: string | null = null;
  let themePreference: 'auto' | 'light' | 'dark' = 'auto';
  let themeToggleIcon = 'fa-moon';
  let themeToggleLabel = 'Switch to dark mode';
  let selectedDirectories: Set<string> = new Set();

const applyTheme = (mode: 'light' | 'dark') => {
  if (typeof document !== 'undefined') {
    document.documentElement.classList.toggle('dark', mode === 'dark');
  }
  isDarkMode = mode === 'dark';
};

const persistThemePreference = (mode: 'light' | 'dark') => {
  themePreference = mode;
  if (typeof window !== 'undefined') {
    window.localStorage.setItem(THEME_STORAGE_KEY, mode);
  }
  applyTheme(mode);
};

const clearThemePreference = () => {
  themePreference = 'auto';
  if (typeof window !== 'undefined') {
    window.localStorage.removeItem(THEME_STORAGE_KEY);
  }
};

const restoreSystemTheme = () => {
  clearThemePreference();
  const prefersDark = systemMediaQuery?.matches ?? false;
  applyTheme(prefersDark ? 'dark' : 'light');
};

  type KindMeta = { label: string; icon: string };
  const KIND_META: Record<SlideIndexItem['kind'], KindMeta> = {
    pptx: { label: 'PPTX', icon: 'fa-file-powerpoint text-orange-500' },
    ppt: { label: 'PPT', icon: 'fa-file-powerpoint text-orange-500' },
    pdf: { label: 'PDF', icon: 'fa-file-pdf text-rose-500' }
  };

  const getKindMeta = (kind: SlideIndexItem['kind']): KindMeta => KIND_META[kind];

  type BadgeColor = 'blue' | 'green' | 'red' | 'yellow' | 'indigo' | 'purple' | 'pink' | 'primary' | 'dark' | 'none';
  type DocumentTypeMeta = { label: string; icon: string; color: BadgeColor };
  const DOCUMENT_TYPE_META: Record<'presentation' | 'book', DocumentTypeMeta> = {
    presentation: { label: 'Presentation', icon: 'fa-presentation-screen', color: 'blue' as BadgeColor },
    book: { label: 'Book', icon: 'fa-book', color: 'green' as BadgeColor }
  };

  const getDocumentTypeMeta = (docType: 'presentation' | 'book' | undefined): DocumentTypeMeta | null => {
    if (!docType) return null;
    return DOCUMENT_TYPE_META[docType] || null;
  };

  const handleDocumentTypeFilterChange = (filter: 'all' | 'presentation' | 'book') => {
    documentTypeFilter = filter;
    
    // Refresh search results with new filter
    if (hasActiveQuery) {
      const trimmed = query.trim();
      if (isOfflineMode) {
        filteredItems = filterLocalItems(trimmed);
      } else {
        startSearch(trimmed, true);
      }
    } else {
      // If no search query, show all items filtered by document type
      hasActiveQuery = true;
      query = '';
      if (isOfflineMode) {
        filteredItems = filterLocalItems('');
      } else {
        startSearch('', true);
      }
    }
  };

  onMount(async () => {
  if (typeof window !== 'undefined') {
    syncViewFromHash();
    const handler = () => syncViewFromHash();
    window.addEventListener('hashchange', handler);
    removeHashListener = () => window.removeEventListener('hashchange', handler);

    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    systemMediaQuery = mediaQuery;
    const storedTheme = window.localStorage.getItem(THEME_STORAGE_KEY);
    if (storedTheme === 'light' || storedTheme === 'dark') {
      themePreference = storedTheme;
      applyTheme(storedTheme);
    } else {
        themePreference = 'auto';
        applyTheme(mediaQuery.matches ? 'dark' : 'light');
      }
      const themeHandler = (event: MediaQueryListEvent) => {
        if (themePreference === 'auto') {
          applyTheme(event.matches ? 'dark' : 'light');
        }
      };
      mediaQuery.addEventListener('change', themeHandler);
      removeThemeListener = () => mediaQuery.removeEventListener('change', themeHandler);
    }
    try {
      const { listen } = await import('@tauri-apps/api/event');
      const unlisten = await listen<{ path: string | null; status: string | null; debugInfo: string | null }>('scan-progress', (event) => {
        // Ignore events if scan was stopped
        if (scanStopped) {
          return;
        }
        
        const newPath = event.payload?.path ?? null;
        const newStatus = event.payload?.status ?? null;
        const newDebugInfo = event.payload?.debugInfo ?? null;
        
        // If it's a new file, start timer
        if (newPath && newPath !== currentScanPath) {
          currentFileStartTime = Date.now();
          processingTime = 0;
          
          // Clear existing interval
          if (processingTimeInterval) {
            clearInterval(processingTimeInterval);
          }
          
          // Start new interval to update processing time
          processingTimeInterval = setInterval(() => {
            if (currentFileStartTime) {
              processingTime = Math.floor((Date.now() - currentFileStartTime) / 1000);
            }
          }, 1000);
        }
        
        // Only update if new values are provided (don't clear on null from backend)
        if (newPath) {
          currentScanPath = newPath;
        }
        if (newStatus) {
          currentScanStatus = newStatus;
        }
        if (newDebugInfo) {
          currentDebugInfo = newDebugInfo;
        }
        
        // Only clear everything if backend explicitly sends all nulls (scan complete)
        if (newPath === null && newStatus === null && newDebugInfo === null) {
          if (processingTimeInterval) {
            clearInterval(processingTimeInterval);
            processingTimeInterval = null;
          }
          currentFileStartTime = null;
          processingTime = 0;
          // Don't clear path/status/debug here - let the scan completion handler do it
        }
      });
      removeScanListener = () => {
        unlisten();
      };
    } catch {
      removeScanListener = null;
    }

    await loadState();
  });

  onDestroy(() => {
    if (searchDebounce) {
      clearTimeout(searchDebounce);
    }
    if (processingTimeInterval) {
      clearInterval(processingTimeInterval);
    }
    removeHashListener?.();
    removeThemeListener?.();
    removeScanListener?.();
    systemMediaQuery = null;
  });

$: {
  themeToggleIcon = isDarkMode ? 'fa-sun' : 'fa-moon';
  themeToggleLabel = isDarkMode ? 'Switch to light mode' : 'Switch to dark mode';
}

  const setView = (next: View) => {
    view = next;
    if (typeof window !== 'undefined') {
      const baseUrl = window.location.pathname + window.location.search;
      if (next === 'help') {
        window.location.hash = '#help';
      } else {
        window.history.replaceState(null, '', baseUrl);
      }
    }
  };

  const syncViewFromHash = () => {
    if (typeof window === 'undefined') {
      return;
    }
    const hash = window.location.hash.replace(/^#/, '');
    view = hash === 'help' ? 'help' : 'index';
  };

  const enableDirectory = (node: HTMLInputElement) => {
    node.setAttribute('webkitdirectory', '');
    node.setAttribute('directory', '');
    node.setAttribute('multiple', 'true');
    return {
      destroy: () => {
        node.removeAttribute('webkitdirectory');
        node.removeAttribute('directory');
        node.removeAttribute('multiple');
      }
    };
  };

  const opensetView = (next: View) => {
    view = next;
    if (typeof window !== 'undefined') {
      const baseUrl = window.location.pathname + window.location.search;
      if (next === 'help') {
        window.location.hash = '#help';
      } else {
        window.history.replaceState(null, '', baseUrl);
      }
    }
  };

  const loadState = async () => {
    isLoading = true;
    loadError = null;
    offlineNotice = null;
    scanErrors = [];
    // ensure we're showing the index view once data is ready
    opensetView('index');
    pdfExtractionWarning = false;
    try {
      const state = await fetchState();
      isOfflineMode = false;
      directories = state.directories;
      // Initialize all directories as selected
      selectedDirectories = new Set(state.directories);
      const sanitisedItems = (state.items ?? []).map(sanitizeItem);
      items = sanitisedItems;
      filteredItems = [];
      lastIndexedAt = state.lastIndexedAt;
      scanErrors = state.warnings ?? [];
      currentScanPath = null;
      query = '';
      hasActiveQuery = false;
      searchError = null;
    } catch (error) {
      isOfflineMode = true;
      offlineNotice =
        'Indexing server unreachable. Running in browser-only mode. Re-link folders whenever slides change.';
      try {
        await loadOfflineState();
      } catch (offlineError) {
        loadError = (offlineError as Error).message;
      }
    } finally {
      isLoading = false;
    }
  };

  const loadOfflineState = async () => {
    const stored = (await localforage.getItem<AppState>(STORE_KEY)) ?? DEFAULT_STATE;
    pdfExtractionWarning = false;
    directories = stored.directories ?? [];
    // Initialize all directories as selected
    selectedDirectories = new Set(stored.directories ?? []);
    const sanitisedItems = (stored.items ?? []).map(sanitizeItem);
    items = sanitisedItems;
    filteredItems = [];
    lastIndexedAt = stored.lastIndexedAt ?? null;
    scanErrors = stored.warnings ?? [];
    currentScanPath = null;
    query = '';
    hasActiveQuery = false;
    searchError = null;
  };

  const filterLocalItems = (raw: string): SlideIndexItem[] => {
    const parsed = parseSearchQuery(raw);
    let results = items;
    
    // Filter by selected directories
    if (selectedDirectories.size === 0) {
      // No directories selected = no results
      return [];
    } else if (selectedDirectories.size < directories.length) {
      // Some directories selected = filter by those
      results = results.filter((item) => {
        const itemPath = item.path || '';
        const itemDir = itemPath.split('/')[0] ?? itemPath;
        
        // Check if item matches any selected directory
        for (const directory of selectedDirectories) {
          const directoryName = directory.split('/').filter(Boolean).pop() ?? directory;
          if (itemDir === directoryName || itemPath.startsWith(directory) || itemDir === directory) {
            return true;
          }
        }
        return false;
      });
    }
    // If all directories selected, don't filter (show all)
    
    // Filter by document type
    if (documentTypeFilter !== 'all') {
      results = results.filter((item) => item.documentType === documentTypeFilter);
    }
    
    // Filter by search query
    if (parsed.isEmpty) {
      return results;
    }
    return results.filter((item) => matchesSearchQuery(item, parsed));
  };

  const executeRemoteSearch = async (trimmed: string, currentToken: number) => {
    try {
      const result = await searchIndex(trimmed);
      if (searchToken !== currentToken) {
        return;
      }
      let sanitised = (result.items ?? []).map(sanitizeItem);
      
      // Filter by selected directories
      if (selectedDirectories.size === 0) {
        // No directories selected = no results
        sanitised = [];
      } else if (selectedDirectories.size < directories.length) {
        // Some directories selected = filter by those
        sanitised = sanitised.filter((item) => {
          const itemPath = item.path || '';
          const itemDir = itemPath.split('/')[0] ?? itemPath;
          
          // Check if item matches any selected directory
          for (const directory of selectedDirectories) {
            const directoryName = directory.split('/').filter(Boolean).pop() ?? directory;
            if (itemDir === directoryName || itemPath.startsWith(directory) || itemDir === directory) {
              return true;
            }
          }
          return false;
        });
      }
      // If all directories selected, don't filter (show all)
      
      // Filter by document type
      if (documentTypeFilter !== 'all') {
        sanitised = sanitised.filter((item) => item.documentType === documentTypeFilter);
      }
      
      filteredItems = sanitised;
      lastIndexedAt = result.lastIndexedAt;
      searchError = null;
    } catch (error) {
      if (searchToken !== currentToken) {
        return;
      }
      searchError = (error as Error).message;
    } finally {
      if (searchToken === currentToken) {
        isSearching = false;
        searchDebounce = null;
      }
    }
  };

  const startSearch = (trimmed: string, immediate = false) => {
    const currentToken = ++searchToken;
    
    // Clear any pending debounce if immediate
    if (immediate && searchDebounce) {
      clearTimeout(searchDebounce);
      searchDebounce = null;
    }
    
    if (isOfflineMode) {
      filteredItems = filterLocalItems(trimmed);
      searchError = null;
      isSearching = false;
      return;
    }
    
    isSearching = true;
    searchError = null;
    void executeRemoteSearch(trimmed, currentToken);
  };

  const handleSearchChange = (value: string) => {
    query = value;
    if (searchDebounce) {
      clearTimeout(searchDebounce);
      searchDebounce = null;
    }
    const trimmed = value.trim();
    hasActiveQuery = trimmed.length > 0;
    if (!trimmed) {
      filteredItems = [];
      searchError = null;
      isSearching = false;
      return;
    }
    // Search only on Enter key or button click, not automatically
  };

  const handleSearchSubmit = () => {
    const trimmed = query.trim();
    hasActiveQuery = trimmed.length > 0;
    if (!trimmed) {
      filteredItems = [];
      searchError = null;
      isSearching = false;
      return;
    }
    startSearch(trimmed, true);
  };

  const handleClearSearch = () => {
    handleSearchChange('');
  };

  const saveOfflineState = async () => {
    const payload: AppState = {
      directories,
      items,
      lastIndexedAt,
      warnings: scanErrors
    };
    await localforage.setItem(STORE_KEY, payload);
  };

  const handleLinkClick = async () => {
    if (isOfflineMode) {
      folderInput?.click();
      return;
    }

    const selections = await selectDirectories();
    if (Array.isArray(selections)) {
      if (selections.length) {
        const merged = Array.from(new Set([...directories, ...selections]));
        await submitDirectories(merged);
      }
      return;
    }

    setView('index');
    handleModalOpen();
  };

  const handleModalOpen = () => {
    newDirectory = '';
    showDirectoryModal = true;
  };

  const handleModalClose = () => {
    showDirectoryModal = false;
    newDirectory = '';
  };

  const saveDirectory = async () => {
    const trimmed = newDirectory.trim();
    if (!trimmed) {
      return;
    }
    const updated = Array.from(new Set([...directories, trimmed]));
    showDirectoryModal = false;
    await submitDirectories(updated);
  };

  const removeDirectory = async (target: string) => {
    const updated = directories.filter((dir) => dir !== target);
    await submitDirectories(updated);
  };

  const submitDirectories = async (updated: string[], autoScan: boolean = true) => {
    console.log('submitDirectories called with:', updated);
    
    if (isOfflineMode) {
      directories = updated;
      // Update selected directories to include only still-existing directories
      selectedDirectories = new Set(Array.from(selectedDirectories).filter(dir => updated.includes(dir)));
      // If no directories are selected, select all
      if (selectedDirectories.size === 0) {
        selectedDirectories = new Set(updated);
      }
      const allowed = new Set(updated);
      if (allowed.size) {
        items = items.filter((item) => allowed.has(item.path.split('/')[0] ?? item.path));
      } else {
        items = [];
      }
      filteredItems = query.trim() ? filterLocalItems(query.trim()) : [];
      await saveOfflineState();
      return;
    }

    // Save directories without scanning
    scanErrors = [];
    try {
      console.log('Calling updateDirectories API with:', updated);
      const summary = await updateDirectories(updated);
      console.log('updateDirectories returned summary:', summary);
      
      directories = updated;
      console.log('Set local directories to:', directories);
      
      // Update selected directories to include only still-existing directories
      selectedDirectories = new Set(Array.from(selectedDirectories).filter(dir => updated.includes(dir)));
      // If no directories are selected, select all
      if (selectedDirectories.size === 0) {
        selectedDirectories = new Set(updated);
      }
      lastIndexedAt = summary.lastIndexedAt;
      if (summary.errors.length) {
        scanErrors = summary.errors;
      }
      
      console.log('Directories saved successfully');
    } catch (error) {
      console.error('Error in submitDirectories:', error);
      scanErrors = [`Failed to update directories: ${(error as Error).message}`];
    }
  };

  const rescanAll = async () => {
    if (isOfflineMode) {
      folderInput?.click();
      return;
    }
    rescanningDirectory = null;
    isRescanning = true;
    scanStopped = false;
    scanErrors = [];
    lastScanSummary = null;
    try {
      const summary = await rescanAllApi();
      
      // If user stopped scan, don't update state
      if (scanStopped) {
        await loadState();
        return;
      }
      
      lastIndexedAt = summary.lastIndexedAt;
      if (summary.errors.length) {
        scanErrors = summary.errors;
      }
      // Set scan summary
      if (summary.scanned !== undefined && summary.cached !== undefined) {
        lastScanSummary = {
          total: summary.scanned + summary.cached,
          scanned: summary.scanned,
          cached: summary.cached
        };
      }
      await loadState();
      if (!scanErrors.length && summary.errors.length) {
        scanErrors = summary.errors;
      }
    } catch (error) {
      if (!scanStopped) {
        scanErrors = [`Rescan failed: ${(error as Error).message}`];
      }
    } finally {
      isRescanning = false;
      scanStopped = false;
      // Clear scan progress after scan completes
      setTimeout(() => {
        currentScanPath = null;
        currentScanStatus = null;
        currentDebugInfo = null;
        if (processingTimeInterval) {
          clearInterval(processingTimeInterval);
          processingTimeInterval = null;
        }
        currentFileStartTime = null;
        processingTime = 0;
      }, 2000); // Show last file for 2 seconds
    }
  };

  const clearCache = async () => {
    if (isOfflineMode) {
      // Clear browser cache
      await localforage.clear();
      items = [];
      filteredItems = [];
      directories = [];
      lastIndexedAt = null;
      scanErrors = [];
      query = '';
      hasActiveQuery = false;
      searchError = null;
      return;
    }
    
    // For server mode, clear the server cache
    try {
      await clearCacheApi();
      await loadState();
    } catch (error) {
      scanErrors = [`Failed to clear cache: ${(error as Error).message}`];
    }
  };

  const handleFolderSelect = async (event: Event) => {
    if (!isOfflineMode) {
      return;
    }
    const input = event.target as HTMLInputElement;
    const files = Array.from(input.files ?? []) as FileWithPath[];
    if (!files.length) {
      return;
    }

    isRescanning = true;
    scanErrors = [];
    try {
      const { newItems, directoryNames, errors } = await processOfflineFiles(files);
      if (!newItems.length && !errors.length) {
        scanErrors = ['No supported slide decks found in the selected folder.'];
      } else {
        const merged = new Map(items.map((item) => [item.id, item]));
        for (const item of newItems) {
          merged.set(item.id, item);
        }
        const mergedValues = Array.from(merged.values()).map(sanitizeItem);
        items = mergedValues.sort((a, b) => (b.updatedAt ?? 0) - (a.updatedAt ?? 0));
        filteredItems = query.trim() ? filterLocalItems(query.trim()) : [];
        const directorySet = new Set([...directories, ...directoryNames]);
        directories = Array.from(directorySet.values()).sort((a, b) => a.localeCompare(b));
        lastIndexedAt = Date.now();
        await saveOfflineState();
        scanErrors = errors;
      }
    } catch (error) {
      scanErrors = [`Failed to process folder: ${(error as Error).message}`];
    } finally {
      if (input) {
        input.value = '';
      }
      isRescanning = false;
    }
  };

  const rescanDirectory = async (directory: string) => {
    if (isOfflineMode) {
      folderInput?.click();
      return;
    }
    isRescanning = true;
    rescanningDirectory = directory;
    scanStopped = false;
    scanErrors = [];
    lastScanSummary = null;
    try {
      const summary = await rescanDirectoryApi(directory);
      
      // If user stopped scan, don't update state
      if (scanStopped) {
        await loadState();
        return;
      }
      
      lastIndexedAt = summary.lastIndexedAt;
      if (summary.errors.length) {
        scanErrors = summary.errors;
      }
      // Set scan summary
      if (summary.scanned !== undefined && summary.cached !== undefined) {
        lastScanSummary = {
          total: summary.scanned + summary.cached,
          scanned: summary.scanned,
          cached: summary.cached
        };
      }
      await loadState();
      if (!scanErrors.length && summary.errors.length) {
        scanErrors = summary.errors;
      }
    } catch (error) {
      if (!scanStopped) {
        scanErrors = [`Rescan failed: ${(error as Error).message}`];
      }
    } finally {
      rescanningDirectory = null;
      isRescanning = false;
      scanStopped = false;
      // Clear scan progress after scan completes
      setTimeout(() => {
        currentScanPath = null;
        currentScanStatus = null;
        currentDebugInfo = null;
        if (processingTimeInterval) {
          clearInterval(processingTimeInterval);
          processingTimeInterval = null;
        }
        currentFileStartTime = null;
        processingTime = 0;
      }, 2000); // Show last file for 2 seconds
    }
  };

  const stripBinaryArtifacts = (input: string): string =>
    input.replace(/[\u0000-\u001f\u007f-\u009f\uFFFD]+/g, ' ');

  const stripXmlTags = (input: string): string => input.replace(/<[^>]+>/g, ' ');

  const processOfflineFiles = async (files: FileWithPath[]) => {
    const entries = files.map((file) => {
      const relativePath = (file.webkitRelativePath ?? file.name).replace(/\\/g, '/');
      const [root] = relativePath.split('/');
      return {
        file,
        relativePath,
        root: root ?? relativePath
      };
    });

    const directoryNames = new Set<string>();
    const newItems: SlideIndexItem[] = [];
    const errors: string[] = [];
    pdfExtractionWarning = false;

    for (const { file, relativePath, root } of entries) {
      const lower = relativePath.toLowerCase();
      const filename = relativePath.split('/').pop() ?? relativePath;
      if (filename.startsWith('~$')) {
        continue;
      }
      try {
        if (lower.endsWith('.pptx')) {
          const indexed = await indexPptxFile(file, relativePath);
          newItems.push(indexed);
          directoryNames.add(root);
        } else if (lower.endsWith('.ppt')) {
          const indexed = await indexLegacyPptFile(file, relativePath);
          newItems.push(indexed);
          directoryNames.add(root);
        } else if (lower.endsWith('.pdf')) {
          const indexed = await indexPdfFile(file, relativePath);
          newItems.push(indexed);
          directoryNames.add(root);
        }
      } catch (error) {
        errors.push(`Failed to inspect ${relativePath}: ${(error as Error).message}`);
      }
    }

    return {
      newItems,
      directoryNames: Array.from(directoryNames.values()),
      errors: pdfExtractionWarning
        ? [...errors, 'Some PDFs do not expose extractable text. Run the Node indexer or OCR for full extraction.']
        : errors
    };
  };

  const indexPptxFile = async (file: FileWithPath, relativePath: string): Promise<SlideIndexItem> => {
    const buffer = await file.arrayBuffer();
    const archive = await JSZip.loadAsync(buffer);
    const slideEntries = Object.keys(archive.files).filter(
      (entry) => entry.startsWith('ppt/slides/slide') && entry.endsWith('.xml')
    );

    let text = '';
    const slidePreviews: SlidePreview[] = [];
    for (const entry of slideEntries) {
      const xmlContent = await archive.files[entry].async('string');
      const slideText = cleanupWhitespace(
        filterNoiseTokens(stripBinaryArtifacts(stripXmlTags(extractSlideText(xmlContent))))
      );
      if (slideText) {
        slidePreviews.push({ index: slidePreviews.length + 1, text: slideText });
        text += ` ${slideText}`;
      }
    }

    const cleaned = cleanupWhitespace(stripBinaryArtifacts(stripXmlTags(text)));
    const keywords = deriveKeywords(cleaned, slidePreviews);

    return {
      id: relativePath,
      path: relativePath,
      name: file.name,
      kind: 'pptx',
      slideCount: slideEntries.length || null,
      snippet: cleaned.slice(0, MAX_SNIPPET_LENGTH),
      keywords,
      createdAt: file.lastModified || Date.now(),
      updatedAt: Date.now(),
      slides: slidePreviews
    };
  };

  const indexPdfFile = async (file: FileWithPath, relativePath: string): Promise<SlideIndexItem> => {
    const arrayBuffer = await file.arrayBuffer();
    const data = new Uint8Array(arrayBuffer);
    const { text, pageCount, skippedCompressed, pages } = await extractPdfContentsBrowser(data);
    if (skippedCompressed) {
      pdfExtractionWarning = true;
    }
    const combined = cleanupWhitespace(
      filterNoiseTokens(stripBinaryArtifacts(stripXmlTags(text)))
    );
    let encounteredText = false;
    const slidePreviews = (pages.length ? pages : [combined])
      .map((pageText, index) => {
        const sanitised = cleanupWhitespace(
          filterNoiseTokens(stripBinaryArtifacts(stripXmlTags(pageText)))
        );
        if (!sanitised || !hasMeaningfulText(sanitised)) {
          return null;
        }
        encounteredText = true;
        return {
          index: index + 1,
          text: sanitised
        };
      })
      .filter((preview): preview is SlidePreview => Boolean(preview));

    if (!encounteredText) {
      pdfExtractionWarning = true;
    }
    const effectiveSnippet =
      slidePreviews.length && hasMeaningfulText(combined) ? combined : '';
    const keywords = deriveKeywords(effectiveSnippet, slidePreviews);

    return {
      id: relativePath,
      path: relativePath,
      name: file.name,
      kind: 'pdf',
      slideCount: pageCount,
      snippet: effectiveSnippet.slice(0, MAX_SNIPPET_LENGTH),
      keywords,
      createdAt: file.lastModified || Date.now(),
      updatedAt: Date.now(),
      slides: slidePreviews
    };
  };

  const indexLegacyPptFile = async (file: FileWithPath, relativePath: string): Promise<SlideIndexItem> => {
    const arrayBuffer = await file.arrayBuffer();
    const ascii = latin1Decoder
      .decode(new Uint8Array(arrayBuffer))
      .replace(/[\u0000-\u0008\u000B\u000C\u000E-\u001F\u007F]+/g, ' ');
    const cleaned = cleanupWhitespace(
      filterNoiseTokens(stripBinaryArtifacts(stripXmlTags(ascii)))
    );
    const slidePreviews = cleaned && hasMeaningfulText(cleaned)
      ? [
          {
            index: 1,
            text: cleaned
          }
        ]
      : [];
    const effectiveSnippet = slidePreviews.length ? cleaned : '';
    const keywords = deriveKeywords(effectiveSnippet, slidePreviews);
    if (!slidePreviews.length) {
      pdfExtractionWarning = true;
    }

    return {
      id: relativePath,
      path: relativePath,
      name: file.name,
      kind: 'ppt',
      slideCount: null,
      snippet: effectiveSnippet.slice(0, MAX_SNIPPET_LENGTH),
      keywords,
      createdAt: file.lastModified || Date.now(),
      updatedAt: Date.now(),
      slides: slidePreviews
    };
  };

  const handleOpenDeck = async (item: SlideIndexItem) => {
    if (isOfflineMode) {
      openDeckError = 'Opening decks is disabled in offline mode.';
      return;
    }
    isOpeningDeck = true;
    openDeckError = null;
    try {
      await openSlideDeck(item.id);
    } catch (error) {
      openDeckError = (error as Error).message;
    } finally {
      isOpeningDeck = false;
    }
  };

  const handlePreview = (item: SlideIndexItem) => {
    selectedItem = item;
    selectedSlides = item.slides ?? [];
    showPreviewModal = true;
  };

  const closePreview = () => {
    showPreviewModal = false;
    selectedItem = null;
    selectedSlides = [];
  };

  const highlightSearchTerms = (text: string, searchQuery: string): string => {
    if (!searchQuery.trim()) {
      return escapeHtml(text);
    }
    
    const parsed = parseSearchQuery(searchQuery);
    if (parsed.isEmpty) {
      return escapeHtml(text);
    }
    
    // Collect all matches with their positions
    const matches: Array<{ start: number; end: number; text: string }> = [];
    const lowerText = text.toLowerCase();
    
    // Find phrase matches
    for (const phrase of parsed.phrases) {
      let pos = 0;
      while ((pos = lowerText.indexOf(phrase, pos)) !== -1) {
        matches.push({
          start: pos,
          end: pos + phrase.length,
          text: text.substring(pos, pos + phrase.length)
        });
        pos += phrase.length;
      }
    }
    
    // Find term matches (whole word boundaries)
    for (const term of parsed.terms) {
      const regex = new RegExp(`\\b${term.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}\\b`, 'gi');
      let match: RegExpExecArray | null;
      while ((match = regex.exec(text)) !== null) {
        matches.push({
          start: match.index,
          end: match.index + match[0].length,
          text: match[0]
        });
      }
    }
    
    // Find wildcard matches
    for (const wildcard of parsed.wildcards) {
      // Extract the pattern without the leading/trailing .* added by wildcardToRegExp
      const wildcardRegex = new RegExp(wildcard.source.slice(2, -2), 'gi');
      let match: RegExpExecArray | null;
      while ((match = wildcardRegex.exec(text)) !== null) {
        matches.push({
          start: match.index,
          end: match.index + match[0].length,
          text: match[0]
        });
      }
    }
    
    if (matches.length === 0) {
      return escapeHtml(text);
    }
    
    // Sort and merge overlapping matches
    matches.sort((a, b) => a.start - b.start);
    const merged: Array<{ start: number; end: number }> = [];
    
    for (const match of matches) {
      if (merged.length === 0 || match.start > merged[merged.length - 1].end) {
        merged.push({ start: match.start, end: match.end });
      } else {
        merged[merged.length - 1].end = Math.max(merged[merged.length - 1].end, match.end);
      }
    }
    
    // Build highlighted HTML
    let result = '';
    let lastIndex = 0;
    
    for (const match of merged) {
      result += escapeHtml(text.substring(lastIndex, match.start));
      result += `<mark class="bg-yellow-200 dark:bg-yellow-600/50">${escapeHtml(text.substring(match.start, match.end))}</mark>`;
      lastIndex = match.end;
    }
    
    result += escapeHtml(text.substring(lastIndex));
    return result;
  };

  const escapeHtml = (text: string): string => {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
  };

  const toggleDirectory = (directory: string) => {
    if (selectedDirectories.has(directory)) {
      selectedDirectories.delete(directory);
    } else {
      selectedDirectories.add(directory);
    }
    selectedDirectories = selectedDirectories; // Trigger reactivity
    
    // Re-apply search filter if there's an active query
    if (hasActiveQuery) {
      const trimmed = query.trim();
      if (isOfflineMode) {
        filteredItems = filterLocalItems(trimmed);
      } else {
        startSearch(trimmed, true);
      }
    }
  };

  const toggleAllDirectories = () => {
    if (selectedDirectories.size === directories.length) {
      selectedDirectories.clear();
    } else {
      selectedDirectories = new Set(directories);
    }
    selectedDirectories = selectedDirectories; // Trigger reactivity
    
    // Re-apply search filter if there's an active query
    if (hasActiveQuery) {
      const trimmed = query.trim();
      if (isOfflineMode) {
        filteredItems = filterLocalItems(trimmed);
      } else {
        startSearch(trimmed, true);
      }
    }
  };

  const formatTimestamp = (timestamp: number | null): string => {
    if (!timestamp) {
      return 'Never';
    }
    return new Date(timestamp).toLocaleString();
  };

  // Calculate cache statistics for a directory
  const getDirectoryStats = (directory: string): { wordCount: number; cacheSize: string } => {
    // Extract the last component of the directory path for matching
    // Directory might be "/Volumes/STORAGE/MEGA/LECTURING" but items have "LECTURING/file.pptx"
    const directoryName = directory.split('/').filter(Boolean).pop() ?? directory;
    
    const directoryItems = items.filter((item) => {
      const itemPath = item.path || '';
      const itemDir = itemPath.split('/')[0] ?? itemPath;
      
      // Match either the full directory path or just the directory name
      return itemDir === directoryName || itemPath.startsWith(directory) || itemDir === directory;
    });

    // Count words from all text content
    let totalWords = 0;
    for (const item of directoryItems) {
      // Count words in snippet
      if (item.snippet) {
        totalWords += item.snippet.split(/\s+/).filter(Boolean).length;
      }
      // Count words in all slides
      if (item.slides) {
        for (const slide of item.slides) {
          if (slide.text) {
            totalWords += slide.text.split(/\s+/).filter(Boolean).length;
          }
        }
      }
      // Count words in keywords
      if (item.keywords) {
        for (const keyword of item.keywords) {
          if (keyword) {
            totalWords += 1;
          }
        }
      }
    }

    // Calculate cache size (approximate JSON size in bytes)
    const cacheJson = JSON.stringify(directoryItems);
    const cacheSizeBytes = new Blob([cacheJson]).size;
    
    // Format size
    let cacheSize: string;
    if (cacheSizeBytes < 1024) {
      cacheSize = `${cacheSizeBytes} B`;
    } else if (cacheSizeBytes < 1024 * 1024) {
      cacheSize = `${(cacheSizeBytes / 1024).toFixed(2)} KB`;
    } else if (cacheSizeBytes < 1024 * 1024 * 1024) {
      cacheSize = `${(cacheSizeBytes / (1024 * 1024)).toFixed(2)} MB`;
    } else {
      cacheSize = `${(cacheSizeBytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
    }

    return { wordCount: totalWords, cacheSize };
  };

  const extractSlideText = (xml: string): string => {
    const regex = /<a:t[^>]*>([\s\S]*?)<\/a:t>/g;
    const segments: string[] = [];
    let match: RegExpExecArray | null;
    while ((match = regex.exec(xml)) !== null) {
      const decoded = decodeXmlEntities(match[1] ?? '');
      if (decoded.trim()) {
        segments.push(decoded.trim());
      }
    }
    return segments.join(' ');
  };

  const cleanupWhitespace = (input: string): string => input.replace(/\s+/g, ' ').trim();

  const filterNoiseTokens = (input: string): string => {
    const tokens = input.split(/\s+/).filter(Boolean);
    const filtered = tokens.filter((token) => !isNoiseToken(token));
    return filtered.join(' ');
  };

  const formatCountLabel = (value: number, singular: string, plural: string): string =>
    `${value.toLocaleString()} ${value === 1 ? singular : plural}`;

  const sanitizeItem = (item: SlideIndexItem): SlideIndexItem => {
    const rawSnippet = cleanupWhitespace(
      filterNoiseTokens(stripBinaryArtifacts(stripXmlTags(item.snippet ?? '')))
    );
    const sanitisedSnippet = hasMeaningfulText(rawSnippet) ? rawSnippet : '';
    const sanitisedKeywords = (item.keywords ?? []).filter((keyword) => !isNoiseToken(keyword));
    const sanitisedSlides = (item.slides ?? [])
      .map((slide) => {
        const text = cleanupWhitespace(
          filterNoiseTokens(stripBinaryArtifacts(stripXmlTags(slide.text ?? '')))
        );
        return text && hasMeaningfulText(text)
          ? {
              index: slide.index,
              text
            }
          : null;
      })
      .filter((slide): slide is SlidePreview => Boolean(slide));
    if (!sanitisedSnippet && !sanitisedSlides.length) {
      pdfExtractionWarning = true;
    }
    return {
      ...item,
      snippet: sanitisedSnippet,
      keywords: sanitisedKeywords,
      slides: sanitisedSlides,
      createdAt: item.createdAt ?? item.updatedAt ?? Date.now()
    };
  };

  const isNoiseToken = (token: string): boolean => {
    const stripped = token.replace(/[()]/g, '');
    if (!/[a-zA-Z]/.test(stripped)) {
      return true;
    }
    if (/^[a-z]{2}-[a-z]{2}$/i.test(stripped)) {
      return true;
    }
    const lowered = stripped.toLowerCase();
    if (NOISE_WORDS.has(lowered)) {
      return true;
    }
    return NOISE_PATTERNS.some((pattern) => pattern.test(lowered));
  };

  const NOISE_WORDS = new Set([
    'rectangle',
    'title',
    'subtitle',
    'body',
    'outline',
    'placeholder',
    'arial',
    'calibri',
    'bold',
    'italic',
    'regular'
  ]);

  const NOISE_PATTERNS = [/^latin-\d+$/, /^slide\d*$/, /^text\d*$/];

  const deriveKeywords = (text: string, slides: SlidePreview[]): string[] => {
    const tokens = text.toLowerCase().match(/[a-z0-9]{3,}/g);
    if (!tokens) {
      return [];
    }
    const frequencies = new Map<string, number>();
    for (const token of tokens) {
      frequencies.set(token, (frequencies.get(token) ?? 0) + 1);
    }
    const slideTokens = new Set(
      slides.flatMap((slide) => slide.text.toLowerCase().match(/[a-z0-9]{3,}/g) ?? [])
    );
    return Array.from(frequencies.entries())
      .filter(([keyword]) => !slideTokens.has(keyword))
      .sort((a, b) => b[1] - a[1])
      .slice(0, MAX_KEYWORDS)
      .map(([keyword]) => keyword);
  };

  const extractPdfContentsBrowser = async (
    data: Uint8Array
  ): Promise<{ text: string; pageCount: number | null; skippedCompressed: boolean; pages: string[] }> => {
    const pages = await extractPdfPagesWeb(data);
    return {
      text: pages.join(' '),
      pageCount: pages.length || null,
      skippedCompressed: false,
      pages
    };
  };

</script>

<svelte:head>
  <title>Slides Indexer</title>
</svelte:head>

<div class="min-h-screen bg-gray-50 text-gray-900 transition-colors dark:bg-slate-950 dark:text-slate-100">
  <input
    class="hidden"
    type="file"
    bind:this={folderInput}
    use:enableDirectory
    on:change={handleFolderSelect}
  />

  <header class="sticky top-0 z-20 border-b border-slate-200 bg-white transition-colors dark:border-slate-700 dark:bg-slate-800">
    <div class="flex flex-wrap items-center gap-3 px-4 py-2 text-slate-600 transition-colors dark:text-slate-200">
      <div class="flex items-center gap-2 text-sm font-semibold uppercase tracking-wide">
        <span class="fa-solid fa-layer-group text-slate-400 dark:text-slate-500"></span>
        Slides Indexer
        <Badge size="xs" color="light" class="uppercase tracking-widest text-[10px] dark:bg-slate-800 dark:text-slate-200">
          v{APP_VERSION}
        </Badge>
        {#if isOfflineMode}
          <span class="inline-flex items-center gap-1 rounded-full bg-amber-100 px-2 py-0.5 text-[10px] font-bold uppercase text-amber-700 dark:bg-amber-400/20 dark:text-amber-200">
            <span class="fa-solid fa-wifi-slash"></span>
            Offline
          </span>
        {/if}
      </div>
      <div class="flex-1">
        <SearchInput
          id={HEADER_SEARCH_ID}
          placeholder="Search slides by keyword, phrase, or wildcard…"
          value={query}
          on:change={(event) => handleSearchChange(event.detail)}
          on:clear={handleClearSearch}
          on:submit={handleSearchSubmit}
          isSearching={isSearching}
        />
      </div>
      <div class="flex flex-wrap items-center gap-2">
        <Button
          size="sm"
          color="light"
          class="!px-3 !py-2"
          ariaLabel={themeToggleLabel}
          title={themeToggleLabel}
          on:click={() => {
            const nextMode = isDarkMode ? 'light' : 'dark';
            persistThemePreference(nextMode);
          }}
          on:contextmenu={(event) => {
            event.preventDefault();
            restoreSystemTheme();
          }}
        >
          <span class={`fa-solid ${themeToggleIcon}`}></span>
        </Button>
        <Button
          size="sm"
          color="light"
          on:click={() => setView(view === 'help' ? 'index' : 'help')}
        >
          <span class="fa-solid {view === 'help' ? 'fa-grid-horizontal' : 'fa-circle-question'} me-2"></span>
          {view === 'help' ? 'Back' : 'Help'}
        </Button>
      </div>
    </div>
  </header>

  <main class="space-y-4 px-4 py-6">
    <section class="rounded-lg border border-slate-200 bg-white px-6 py-4 transition-colors dark:border-slate-700 dark:bg-slate-800">
      <h1 class="text-3xl font-semibold text-slate-900 dark:text-slate-100">
        Find any slide in seconds
        <span class="ml-3 inline-flex items-center gap-1 rounded bg-slate-100 px-2 py-1 text-xs font-normal text-slate-600 dark:bg-slate-700 dark:text-slate-400">
          <span class="fa-solid fa-code-branch"></span>
          {APP_VERSION}
        </span>
      </h1>
      <p class="mt-2 text-sm text-slate-600 dark:text-slate-300">
        {formatCountLabel(directories.length, 'linked folder', 'linked folders')} · {formatCountLabel(items.length, 'indexed document', 'indexed documents')} · Last update {formatTimestamp(lastIndexedAt)}
      </p>
      <p class="mt-4 text-sm text-slate-600 dark:text-slate-300">
        Index PowerPoint and PDF decks, preview slides instantly, and launch files in their native apps without digging through folders.
      </p>
      <div class="mt-6 flex flex-wrap items-center gap-2">
        <Button size="sm" class="bg-orange-500 hover:bg-orange-600 dark:bg-orange-500 dark:hover:bg-orange-600" on:click={handleLinkClick}>
          <span class="fa-solid fa-folder-plus me-2"></span>
          {isOfflineMode ? 'Pick folder' : 'Link folder'}
        </Button>
        {#if isRescanning}
          <Button
            size="sm"
            color="red"
            on:click={() => {
              scanStopped = true;
              currentScanPath = null;
              currentScanStatus = null;
              currentDebugInfo = null;
              if (processingTimeInterval) {
                clearInterval(processingTimeInterval);
                processingTimeInterval = null;
              }
              currentFileStartTime = null;
              processingTime = 0;
              scanErrors = ['Scan stopped by user. Files scanned before stopping were saved to cache.'];
            }}
            disabled={scanStopped}
          >
            {#if scanStopped}
              <span class="fa-solid fa-circle-notch fa-spin me-2"></span>
              Stopping…
            {:else}
              <span class="fa-solid fa-stop me-2"></span>
              Stop scan
            {/if}
          </Button>
        {:else}
          <Button
            size="sm"
            color="light"
            on:click={rescanAll}
          >
            <span class="fa-solid fa-rotate me-2"></span>
            Rescan
          </Button>
        {/if}
        <Button
          size="sm"
          color="red"
          on:click={() => { showClearCacheModal = true; }}
          disabled={isRescanning}
        >
          <span class="fa-solid fa-trash me-2"></span>
          Clear Cache
        </Button>
      </div>
      
      {#if isRescanning}
        <div class="mt-3 space-y-3">
          {#if currentScanPath}
            <div class="space-y-2">
              <div class="flex items-center gap-2 text-sm text-slate-600 dark:text-slate-400">
                {#if currentScanStatus === 'cached'}
                  <span class="fa-solid fa-check text-green-500"></span>
                {:else if currentScanStatus === 'ocr'}
                  <span class="fa-solid fa-magnifying-glass text-purple-500" title="Running OCR"></span>
                {:else if currentScanStatus === 'scanning'}
                  <span class="fa-solid fa-xmark text-orange-500"></span>
                {:else}
                  <span class="fa-solid fa-circle-notch fa-spin text-orange-500"></span>
                {/if}
                <span class="break-all font-medium">{currentScanPath}</span>
              </div>
              {#if processingTime > 0}
                <div class="ml-6 flex items-center gap-2 text-xs text-slate-500 dark:text-slate-400">
                  <span class="fa-solid fa-clock"></span>
                  <span>Processing for {processingTime} second{processingTime === 1 ? '' : 's'}...</span>
                  {#if processingTime > 10}
                    <span class="text-amber-600 dark:text-amber-400">(Large file or OCR in progress)</span>
                  {/if}
                </div>
              {/if}
              {#if !currentDebugInfo || !currentDebugInfo.trim()}
                {#if currentScanStatus === 'cached'}
                  <div class="ml-6 text-xs text-green-600 dark:text-green-400">
                    <span class="fa-solid fa-check-circle me-1"></span>
                    File retrieved from cache (no scan needed)
                  </div>
                {:else}
                  <div class="ml-6 text-xs text-slate-400 dark:text-slate-500 italic">
                    <span class="fa-solid fa-info-circle me-1"></span>
                    Processing file...
                  </div>
                {/if}
              {/if}
            </div>
          {:else}
            <div class="flex items-center gap-2 text-sm text-slate-600 dark:text-slate-400">
              <span class="fa-solid fa-circle-notch fa-spin text-orange-500"></span>
              <span class="font-medium">Starting scan...</span>
            </div>
          {/if}
          {#if currentDebugInfo && currentDebugInfo.trim()}
            <div class="ml-6 space-y-2 rounded border-l-4 {currentScanStatus === 'ocr' ? 'border-purple-400 bg-purple-50 dark:border-purple-500 dark:bg-purple-900/20' : 'border-blue-400 bg-blue-50 dark:border-blue-500 dark:bg-blue-900/20'} pl-4 pr-3 py-3">
              <div class="flex items-center gap-2 text-xs font-bold uppercase tracking-wide {currentScanStatus === 'ocr' ? 'text-purple-700 dark:text-purple-400' : 'text-blue-700 dark:text-blue-400'}">
                <span class="fa-solid {currentScanStatus === 'ocr' ? 'fa-magnifying-glass' : 'fa-info-circle'}"></span>
                <span>{currentScanStatus === 'ocr' ? 'OCR Processing' : 'Scan Details'}</span>
              </div>
              <div class="space-y-1.5 text-xs leading-relaxed text-slate-700 dark:text-slate-300">
                {#each currentDebugInfo.split('\n') as line}
                  {#if line.includes('━━━━━━━━━━━━━━━━━━━━━━')}
                    <div class="my-3 border-t-2 border-dashed border-purple-300 dark:border-purple-600"></div>
                  {:else if line.includes('OCR Processing:')}
                    <div class="flex items-start gap-2 font-bold">
                      <span class="fa-solid fa-magnifying-glass text-purple-500 mt-0.5"></span>
                      <span class="text-purple-700 dark:text-purple-300">{line}</span>
                    </div>
                  {:else if line.includes('Extracting text from images')}
                    <div class="flex items-start gap-2">
                      <span class="fa-solid fa-image text-purple-400 mt-0.5"></span>
                      <span class="text-purple-600 dark:text-purple-400">{line}</span>
                    </div>
                  {:else if line.includes('may take a few moments')}
                    <div class="flex items-start gap-2">
                      <span class="fa-solid fa-hourglass-half text-amber-500 mt-0.5"></span>
                      <span class="text-amber-600 dark:text-amber-400 italic">{line}</span>
                    </div>
                  {:else if line.includes('Rescan Information:') || line.includes('New File Detected')}
                    <div class="flex items-start gap-2 font-bold">
                      <span class="fa-solid fa-info-circle text-blue-500 mt-0.5"></span>
                      <span class="text-blue-700 dark:text-blue-300">{line}</span>
                    </div>
                  {:else if line.includes('Cached checksum:')}
                    <div class="flex items-start gap-2">
                      <span class="fa-solid fa-fingerprint text-purple-500 mt-0.5"></span>
                      <span class="text-slate-700 dark:text-slate-300 font-mono text-[11px]">{line}</span>
                    </div>
                  {:else if line.includes('Current checksum:')}
                    <div class="flex items-start gap-2">
                      <span class="fa-solid fa-fingerprint text-indigo-500 mt-0.5"></span>
                      <span class="text-slate-700 dark:text-slate-300 font-mono text-[11px]">{line}</span>
                    </div>
                  {:else if line.includes('Cached mod_time:') || line.includes('Current mod_time:')}
                    <div class="flex items-start gap-2">
                      <span class="fa-solid fa-clock text-blue-400 mt-0.5"></span>
                      <span class="text-slate-600 dark:text-slate-400 text-[11px]">{line}</span>
                    </div>
                  {:else if line.includes('Checksums MATCH') || line.includes('Content unchanged')}
                    <div class="flex items-start gap-2">
                      <span class="fa-solid fa-check-circle text-green-500 mt-0.5"></span>
                      <span class="font-semibold text-green-700 dark:text-green-400">{line}</span>
                    </div>
                  {:else if line.includes('File content CHANGED') || line.includes('Checksum mismatch')}
                    <div class="flex items-start gap-2">
                      <span class="fa-solid fa-exclamation-triangle text-red-500 mt-0.5"></span>
                      <span class="font-semibold text-red-700 dark:text-red-400">{line}</span>
                    </div>
                  {:else if line.includes('No cached checksum') || line.includes('First scan')}
                    <div class="flex items-start gap-2">
                      <span class="fa-solid fa-info-circle text-amber-500 mt-0.5"></span>
                      <span class="text-amber-700 dark:text-amber-400">{line}</span>
                    </div>
                  {:else if line.includes('First time indexing')}
                    <div class="flex items-start gap-2">
                      <span class="fa-solid fa-plus-circle text-teal-500 mt-0.5"></span>
                      <span class="font-semibold text-teal-700 dark:text-teal-400">{line}</span>
                    </div>
                  {:else if line.includes('Checksum:')}
                    <div class="flex items-start gap-2">
                      <span class="fa-solid fa-fingerprint text-slate-500 mt-0.5"></span>
                      <span class="text-slate-600 dark:text-slate-400 font-mono text-[11px]">{line}</span>
                    </div>
                  {:else if line.trim()}
                    <div class="flex items-start gap-2">
                      <span class="fa-solid fa-angle-right text-slate-400 mt-0.5"></span>
                      <span class="text-slate-600 dark:text-slate-400 text-[11px]">{line}</span>
                    </div>
                  {/if}
                {/each}
              </div>
            </div>
          {/if}
        </div>
      {:else if lastScanSummary}
        <div class="mt-3 rounded-lg border border-slate-200 bg-slate-50/50 px-4 py-3 dark:border-slate-700 dark:bg-slate-800/50">
          <div class="flex items-center gap-2 text-sm font-semibold text-slate-700 dark:text-slate-200">
            <span class="fa-solid fa-check-circle text-green-500"></span>
            Scan Complete
          </div>
          <div class="mt-2 grid grid-cols-3 gap-3 text-xs">
            <div>
              <span class="font-semibold text-slate-600 dark:text-slate-300">Total:</span>
              <span class="ml-1 text-slate-700 dark:text-slate-200">{lastScanSummary.total}</span>
            </div>
            <div>
              <span class="font-semibold text-slate-600 dark:text-slate-300">Scanned:</span>
              <span class="ml-1 text-slate-700 dark:text-slate-200">{lastScanSummary.scanned}</span>
            </div>
            <div>
              <span class="font-semibold text-slate-600 dark:text-slate-300">Cached:</span>
              <span class="ml-1 text-slate-700 dark:text-slate-200">{lastScanSummary.cached}</span>
            </div>
          </div>
        </div>
      {/if}
    </section>

    <section class="grid grid-cols-3 gap-3">
      <article class="rounded border border-slate-200 bg-white px-4 py-3 transition-colors dark:border-slate-700 dark:bg-slate-800">
        <span class="text-xs font-semibold uppercase tracking-wide text-slate-500 dark:text-slate-400">Directories</span>
        <p class="mt-2 text-2xl font-semibold text-slate-900 dark:text-slate-100">{directories.length.toLocaleString()}</p>
        <p class="text-sm text-slate-500 dark:text-slate-400">Organise your decks by linking one or many root folders.</p>
      </article>
      <article class="rounded border border-slate-200 bg-white px-4 py-3 transition-colors dark:border-slate-700 dark:bg-slate-800">
        <span class="text-xs font-semibold uppercase tracking-wide text-slate-500 dark:text-slate-400">Indexed decks</span>
        <p class="mt-2 text-2xl font-semibold text-slate-900 dark:text-slate-100">{items.length.toLocaleString()}</p>
        <p class="text-sm text-slate-500 dark:text-slate-400">Each deck is parsed for fast keyword and slide-level search.</p>
      </article>
      <article class="rounded border border-slate-200 bg-white px-4 py-3 transition-colors dark:border-slate-700 dark:bg-slate-800">
        <span class="text-xs font-semibold uppercase tracking-wide text-slate-500 dark:text-slate-400">Last indexed</span>
        <p class="mt-2 text-2xl font-semibold text-slate-900 dark:text-slate-100">{formatTimestamp(lastIndexedAt)}</p>
        <p class="text-sm text-slate-500 dark:text-slate-400">Kick off a rescan any time decks are updated.</p>
      </article>
    </section>

    {#if offlineNotice}
      <Alert color="yellow" class="border border-amber-200 bg-amber-50/80 shadow-sm">
        <span class="font-semibold text-amber-900">Offline mode:</span> {offlineNotice}
      </Alert>
    {/if}

    {#if loadError}
      <Alert color="failure" class="shadow-sm">
        Failed to load index: {loadError}
      </Alert>
    {/if}

    {#if openDeckError}
      <Alert color="failure" class="shadow-sm">
        {openDeckError}
      </Alert>
    {/if}

    {#if scanErrors.length}
      <Alert color="warning" class="shadow-sm">
        <div class="flex flex-col gap-2">
          <span class="font-semibold text-slate-900">Indexer warnings</span>
          <ul class="list-disc ps-5 text-sm text-slate-700">
            {#each scanErrors as message}
              <li>{message}</li>
            {/each}
          </ul>
        </div>
      </Alert>
    {/if}

    {#if pdfExtractionWarning}
      <Alert color="warning" class="shadow-sm">
        Some PDFs do not expose extractable text. For full PDF scanning with OCR support, install poppler and tesseract (e.g., <code class="font-mono">brew install poppler tesseract</code> on macOS) and run the Node indexer server.
      </Alert>
    {/if}

    {#if view === 'help'}
      <div class="rounded-lg border border-slate-200 bg-white px-6 py-6 transition-colors dark:border-slate-700 dark:bg-slate-800">
        <HelpContent />
      </div>
    {:else}
      {#if isLoading}
        <div class="flex justify-center py-20">
          <Spinner size="12" />
        </div>
      {:else}
        <p class="text-sm text-slate-500 dark:text-slate-300">
          Tips: wrap exact phrases in quotes, use `*` or `?` for wildcards, and results update instantly as you type.
        </p>
        <section class="space-y-5">
          <div class="flex items-center justify-between">
            <h2 class="text-lg font-semibold text-slate-800 dark:text-slate-100">Directories</h2>
            {#if directories.length}
              <div class="flex flex-wrap items-center gap-3">
                <Button size="xs" color="light" on:click={toggleAllDirectories}>
                  <span class="fa-solid {selectedDirectories.size === directories.length ? 'fa-square-check' : 'fa-square'} me-2"></span>
                  {selectedDirectories.size === directories.length ? 'Deselect All' : 'Select All'}
                </Button>
                <span class="text-xs text-slate-500 dark:text-slate-300">
                  {selectedDirectories.size} of {directories.length} selected for search
                </span>
              </div>
            {/if}
          </div>
          <div class="rounded-lg border border-slate-200 bg-white transition-colors dark:border-slate-700 dark:bg-slate-800">
            {#if directories.length}
              <ul class="divide-y divide-slate-200 dark:divide-slate-700">
                {#each directories as directory}
                  {@const stats = getDirectoryStats(directory)}
                  <li class="flex flex-wrap items-center justify-between gap-3 px-5 py-4 transition hover:bg-slate-50 dark:hover:bg-slate-800/80">
                    <div class="flex min-w-0 flex-1 items-center gap-3">
                      <button
                        type="button"
                        class="flex-shrink-0 cursor-pointer text-primary-600 hover:text-primary-700 dark:text-primary-400 dark:hover:text-primary-300"
                        on:click={() => toggleDirectory(directory)}
                        aria-label="Toggle directory selection"
                      >
                        <span class="fa-solid {selectedDirectories.has(directory) ? 'fa-square-check' : 'fa-square'} text-xl"></span>
                      </button>
                      <div class="min-w-0 flex-1 space-y-1">
                        <p class="truncate text-sm font-medium text-slate-800 dark:text-slate-100">{directory}</p>
                        <div class="flex flex-wrap items-center gap-3 text-xs text-slate-500 dark:text-slate-400">
                          <span class="inline-flex items-center gap-1.5" title="Total words in cache">
                            <span class="fa-solid fa-font text-slate-400 dark:text-slate-500"></span>
                            {stats.wordCount.toLocaleString()} words
                          </span>
                          <span class="inline-flex items-center gap-1.5" title="Cache size">
                            <span class="fa-solid fa-database text-slate-400 dark:text-slate-500"></span>
                            {stats.cacheSize}
                          </span>
                        </div>
                      </div>
                    </div>
                    <div class="flex flex-wrap gap-2">
                      <Button
                        size="xs"
                        color="light"
                        disabled={isRescanning && rescanningDirectory !== directory}
                        on:click={() => rescanDirectory(directory)}
                      >
                        {#if isRescanning && rescanningDirectory === directory}
                          <div class="flex min-w-36 max-w-80 flex-col items-start gap-1">
                            <div class="h-1 w-full overflow-hidden rounded-full bg-primary-100 dark:bg-primary-900/40">
                              <div class="h-full w-full animate-pulse rounded-full bg-primary-500"></div>
                            </div>
                            <span class="text-xs font-semibold text-primary-600 dark:text-primary-300">Rescanning…</span>
                            {#if currentScanPath}
                              <span class="w-full truncate text-[10px] text-slate-500 dark:text-slate-400">
                                {currentScanPath}
                              </span>
                            {/if}
                          </div>
                        {:else}
                          <span class="fa-solid fa-rotate me-2"></span>
                          Rescan
                        {/if}
                      </Button>
                      <Button size="xs" color="red" on:click={() => removeDirectory(directory)}>
                        <span class="fa-solid fa-xmark me-2"></span>
                        Unlink
                      </Button>
                    </div>
                  </li>
                {/each}
              </ul>
            {:else}
              <div class="px-5 py-6 text-sm text-slate-600 dark:text-slate-300">
                No directories linked yet. Use <strong>Link a folder</strong> to add your slide decks.
              </div>
            {/if}
          </div>
        </section>

        <section class="space-y-4">
          <div class="flex items-center justify-between gap-3">
            <h2 class="text-lg font-semibold text-slate-800 dark:text-slate-100">Results</h2>
            <div class="flex flex-wrap items-center gap-2">
              {#if selectedDirectories.size > 0 && selectedDirectories.size < directories.length}
                <Badge color="blue" class="rounded-full px-3 text-xs uppercase tracking-wide dark:bg-blue-500/20 dark:text-blue-200">
                  <span class="fa-solid fa-filter me-1"></span>
                  {selectedDirectories.size} folder{selectedDirectories.size === 1 ? '' : 's'}
                </Badge>
              {/if}
              <Badge color="blue" class="rounded-full px-3 text-xs uppercase tracking-wide dark:bg-primary-500/20 dark:text-primary-200">
                {hasActiveQuery
                  ? formatCountLabel(filteredItems.length, 'match', 'matches')
                  : 'Ready'}
              </Badge>
            </div>
          </div>

          <!-- Document Type Filter Buttons -->
          <div class="flex flex-wrap items-center gap-2">
            <span class="text-sm text-slate-600 dark:text-slate-400">Document type:</span>
            <Button
              size="xs"
              color={documentTypeFilter === 'all' ? 'blue' : 'light'}
              on:click={() => handleDocumentTypeFilterChange('all')}
              class="!px-3 !py-1.5"
            >
              <span class="fa-solid fa-list me-1"></span>
              All
            </Button>
            <Button
              size="xs"
              color={documentTypeFilter === 'presentation' ? 'blue' : 'light'}
              on:click={() => handleDocumentTypeFilterChange('presentation')}
              class="!px-3 !py-1.5"
            >
              <span class="fa-solid fa-presentation-screen me-1"></span>
              Presentations
            </Button>
            <Button
              size="xs"
              color={documentTypeFilter === 'book' ? 'green' : 'light'}
              on:click={() => handleDocumentTypeFilterChange('book')}
              class="!px-3 !py-1.5"
            >
              <span class="fa-solid fa-book me-1"></span>
              Books
            </Button>
          </div>

          {#if !hasActiveQuery}
            <div class="rounded-lg border border-slate-200 bg-white px-6 py-6 text-sm text-slate-600 transition-colors dark:border-slate-700 dark:bg-slate-800 dark:text-slate-300">
              Start typing in the search bar above to find matching documents.
            </div>
          {:else if !filteredItems.length}
            <div class="rounded-lg border border-slate-200 bg-white px-6 py-6 text-sm text-slate-600 transition-colors dark:border-slate-700 dark:bg-slate-800 dark:text-slate-300">
              {query.trim()
                ? 'No results matched your search. Try removing filters or checking spelling.'
                : 'No documents indexed yet. Link a directory or rescan to populate results.'}
            </div>
          {:else}
            <div class={`auto-rows-fr ${gridClass} grid-cols-3`}>
              {#each filteredItems as item (item.id)}
                <article class="flex h-full flex-col gap-4 rounded-lg border border-slate-200 bg-white p-4 transition hover:bg-slate-50 dark:border-slate-700 dark:bg-slate-800 dark:hover:bg-slate-700/50">
                  <div class="flex items-start justify-between gap-3">
                    <div class="min-w-0 space-y-1">
                      <h3 class="truncate text-lg font-semibold text-slate-900 dark:text-slate-100">{item.name}</h3>
                      <p class="truncate text-xs text-slate-500 dark:text-slate-300">{item.path}</p>
                    </div>
                    <div class="flex flex-wrap gap-2">
                      <Badge color="light" class="whitespace-nowrap dark:bg-slate-800 dark:text-slate-200">
                        <span class={`fa-solid ${getKindMeta(item.kind).icon} me-1`}></span>
                        {getKindMeta(item.kind).label}
                      </Badge>
                      {#if item.documentType}
                        {@const docMeta = getDocumentTypeMeta(item.documentType)}
                        {#if docMeta}
                          <Badge color={docMeta.color} class="whitespace-nowrap">
                            <span class={`fa-solid ${docMeta.icon} me-1`}></span>
                            {docMeta.label}
                          </Badge>
                        {/if}
                      {/if}
                    </div>
                  </div>

                  <div class="flex flex-wrap gap-2 text-xs text-slate-500 dark:text-slate-300">
                    <span class="inline-flex items-center gap-1 rounded-full bg-slate-100 px-2 py-1 dark:bg-slate-800/80">
                      <span class="fa-solid fa-layer-group text-slate-400 dark:text-slate-500"></span>
                      {item.slideCount ?? '—'} {item.documentType === 'book' ? 'page' : 'slide'}{item.slideCount === 1 ? '' : 's'}
                    </span>
                    <span class="inline-flex items-center gap-1 rounded-full bg-slate-100 px-2 py-1 dark:bg-slate-800/80">
                      <span class="fa-solid fa-calendar-plus text-slate-400 dark:text-slate-500"></span>
                      Created {formatTimestamp(item.createdAt ?? null)}
                    </span>
                    <span class="inline-flex items-center gap-1 rounded-full bg-slate-100 px-2 py-1 dark:bg-slate-800/80">
                      <span class="fa-solid fa-clock text-slate-400 dark:text-slate-500"></span>
                      Updated {formatTimestamp(item.updatedAt ?? null)}
                    </span>
                  </div>

                  {#if item.snippet}
                    <p class="text-sm leading-relaxed text-slate-600 max-h-28 overflow-hidden text-ellipsis dark:text-slate-300">
                      {item.snippet}
                    </p>
                  {:else}
                    <p class="text-sm text-slate-500 italic dark:text-slate-400">No text preview available.</p>
                  {/if}

                  {#if item.keywords?.length}
                    <div class="flex flex-wrap gap-1">
                      {#each item.keywords.slice(0, 6) as keyword}
                        <span class="inline-flex items-center rounded-full bg-slate-100 px-2 py-1 text-[11px] uppercase tracking-wide text-slate-600 dark:bg-slate-800/80 dark:text-slate-300">
                          {keyword}
                        </span>
                      {/each}
                    </div>
                  {/if}

                  <div class="mt-auto flex flex-wrap gap-2">
                    <Button size="xs" color="light" on:click={() => handlePreview(item)}>
                      <span class="fa-solid fa-eye me-2"></span>
                      Preview
                    </Button>
                    <Button
                      size="xs"
                      on:click={() => handleOpenDeck(item)}
                      disabled={isOpeningDeck || isOfflineMode}
                    >
                      {#if isOpeningDeck}
                        <Spinner size="3" />
                        <span class="ms-2">Opening…</span>
                      {:else}
                        <span class="fa-solid fa-up-right-from-square me-2"></span>
                        Open in app
                      {/if}
                    </Button>
                  </div>
                </article>
              {/each}
            </div>
          {/if}
        </section>
      {/if}
    {/if}
  </main>

  <Modal bind:open={showDirectoryModal} size="md" on:close={handleModalClose}>
    <div slot="header" class="text-lg font-semibold text-slate-800">
      Link a folder
    </div>
    <div class="space-y-4">
      <Input
        placeholder="Absolute path to folder"
        bind:value={newDirectory}
        on:keydown={(event) => {
          if (event.key === 'Enter') {
            saveDirectory();
          }
        }}
        autofocus
      />
      <p class="text-xs text-slate-500">
        Paths are forwarded to the indexing server. Ensure the server has access to the location.
      </p>
    </div>
    <div slot="footer" class="flex justify-end gap-2">
      <Button color="light" on:click={handleModalClose}>Cancel</Button>
      <Button on:click={saveDirectory}>Save</Button>
    </div>
  </Modal>

  <Modal bind:open={showPreviewModal} size="5xl" on:close={closePreview}>
    <div slot="header" class="flex flex-wrap items-center justify-between gap-3">
      <div>
        <h3 class="text-lg font-semibold text-slate-800">
          {selectedItem?.name ?? 'Preview'}
        </h3>
        <p class="text-sm text-slate-500">{selectedItem?.path}</p>
      </div>
      {#if selectedItem}
        <Badge color="light" class="text-xs uppercase">
          <span class={`fa-solid ${getKindMeta(selectedItem.kind).icon} me-1`}></span>
          {getKindMeta(selectedItem.kind).label}
        </Badge>
      {/if}
    </div>
    <div class="space-y-4 overflow-y-auto pr-1" style="max-height: 70vh;">
      {#if selectedSlides.length}
        {#each selectedSlides as slide}
          <Card size="none" class="shadow-sm">
            <h4 class="text-sm font-semibold text-slate-700 dark:text-slate-200">
              {selectedItem?.documentType === 'book' ? 'Page' : 'Slide'} {slide.index}
            </h4>
            <p class="text-sm leading-relaxed text-slate-600 dark:text-slate-300 whitespace-pre-wrap">
              {@html highlightSearchTerms(slide.text, query)}
            </p>
          </Card>
        {/each}
      {:else if selectedItem?.snippet}
        <p class="text-sm leading-relaxed text-slate-600 dark:text-slate-300 whitespace-pre-wrap">
          {@html highlightSearchTerms(selectedItem.snippet, query)}
        </p>
      {:else}
        <p class="text-sm text-slate-500">
          No preview available for this document.
        </p>
      {/if}
    </div>
    <div slot="footer" class="flex justify-end">
      <Button color="light" on:click={closePreview}>Close</Button>
    </div>
  </Modal>

  <Modal bind:open={showClearCacheModal} size="md" on:close={() => { showClearCacheModal = false; }}>
    <div slot="header" class="text-lg font-semibold text-slate-800 dark:text-slate-100">
      Clear Cache
    </div>
    <div class="space-y-4">
      <p class="text-sm text-slate-600 dark:text-slate-300">
        Are you sure you want to clear the cache? This will remove all indexed data and require a complete rescan of all linked directories.
      </p>
      <div class="rounded border border-amber-200 bg-amber-50 px-4 py-3 dark:border-amber-800 dark:bg-amber-900/30">
        <div class="flex items-start gap-2">
          <span class="fa-solid fa-exclamation-triangle text-amber-600 dark:text-amber-400 mt-0.5"></span>
          <div class="text-sm text-amber-800 dark:text-amber-200">
            <strong>Warning:</strong> All cached checksums, OCR results, and indexed content will be permanently removed. This action cannot be undone.
          </div>
        </div>
      </div>
    </div>
    <div slot="footer" class="flex justify-end gap-2">
      <Button color="light" on:click={() => { showClearCacheModal = false; }}>Cancel</Button>
      <Button color="red" on:click={async () => {
        showClearCacheModal = false;
        await clearCache();
      }}>
        <span class="fa-solid fa-trash me-2"></span>
        Clear Cache
      </Button>
    </div>
  </Modal>
</div>
