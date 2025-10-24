import type { AppState, SlideIndexItem, ScanSummary } from '../../shared/types';

type TauriInvoke = <T>(command: string, args?: Record<string, unknown>) => Promise<T>;

declare global {
  interface Window {
    __TAURI__?: {
      invoke: TauriInvoke;
    };
  }
}

const API_BASE = import.meta.env.VITE_API_BASE?.replace(/\/$/, '') ?? 'http://127.0.0.1:4173/api';

let cachedInvoke: TauriInvoke | null | undefined;
let failedDynamicImport = false;

const TAURI_POLL_ATTEMPTS = 5;
const TAURI_POLL_DELAY_MS = 50;

function isTauriEnvironment(): boolean {
  if (typeof window === 'undefined') {
    return false;
  }
  return '__TAURI_IPC__' in window || typeof window.__TAURI__ !== 'undefined';
}

async function resolveTauriInvoke(): Promise<TauriInvoke | null> {
  if (cachedInvoke) {
    return cachedInvoke;
  }
  if (typeof window === 'undefined') {
    return null;
  }
  const globalInvoke = window.__TAURI__?.invoke;
  if (globalInvoke) {
    cachedInvoke = globalInvoke;
    return cachedInvoke;
  }
  if (!failedDynamicImport && '__TAURI_IPC__' in window) {
    try {
      const { invoke } = await import('@tauri-apps/api/tauri');
      cachedInvoke = invoke;
      return cachedInvoke;
    } catch {
      failedDynamicImport = true;
    }
  }
  return null;
}

async function waitForTauriInvoke(): Promise<TauriInvoke | null> {
  for (let attempt = 0; attempt < TAURI_POLL_ATTEMPTS; attempt += 1) {
    const invoke = await resolveTauriInvoke();
    if (invoke) {
      return invoke;
    }
    if (!isTauriEnvironment()) {
      break;
    }
    await new Promise((resolve) => setTimeout(resolve, TAURI_POLL_DELAY_MS));
  }
  return null;
}

async function request<T>(input: RequestInfo, init?: RequestInit): Promise<T> {
  const response = await fetch(input, {
    ...init,
    headers: {
      'Content-Type': 'application/json',
      ...(init?.headers ?? {})
    }
  });

  if (!response.ok) {
    const message = await response.text();
    throw new Error(message || `Request failed with status ${response.status}`);
  }
  return response.json() as Promise<T>;
}

export async function fetchState(): Promise<AppState> {
  const invoke = await waitForTauriInvoke();
  if (invoke) {
    return invoke<AppState>('fetch_state');
  }
  return request<AppState>(`${API_BASE}/state`);
}

export async function updateDirectories(directories: string[]): Promise<ScanSummary> {
  const invoke = await waitForTauriInvoke();
  if (invoke) {
    return invoke<ScanSummary>('update_directories', { directories });
  }
  return request<ScanSummary>(`${API_BASE}/directories`, {
    method: 'POST',
    body: JSON.stringify({ directories })
  });
}

export async function rescan(): Promise<ScanSummary> {
  const invoke = await waitForTauriInvoke();
  if (invoke) {
    return invoke<ScanSummary>('rescan');
  }
  return request<ScanSummary>(`${API_BASE}/rescan`, {
    method: 'POST'
  });
}

export async function rescanDirectory(directory: string): Promise<ScanSummary> {
  const invoke = await waitForTauriInvoke();
  if (invoke) {
    return invoke<ScanSummary>('rescan_directory', { directory });
  }
  return request<ScanSummary>(`${API_BASE}/rescan`, {
    method: 'POST',
    body: JSON.stringify({ directory })
  });
}

export async function openSlideDeck(id: string): Promise<void> {
  const invoke = await waitForTauriInvoke();
  if (invoke) {
    await invoke('open_slide_deck', { id });
    return;
  }
  await request(`${API_BASE}/open`, {
    method: 'POST',
    body: JSON.stringify({ id })
  });
}

export async function searchIndex(query: string): Promise<{
  items: SlideIndexItem[];
  total: number;
  lastIndexedAt: number | null;
}> {
  const trimmed = query.trim();
  const invoke = await waitForTauriInvoke();
  if (invoke) {
    return invoke('search_index', { query: trimmed.length ? trimmed : null });
  }
  const params = new URLSearchParams();
  if (trimmed) {
    params.set('q', trimmed);
  }
  const suffix = params.toString() ? `?${params.toString()}` : '';
  return request(`${API_BASE}/index${suffix}`);
}

export async function clearCache(): Promise<{ success: boolean; message: string }> {
  const invoke = await waitForTauriInvoke();
  if (invoke) {
    return invoke('clear_cache');
  }
  return request(`${API_BASE}/clear-cache`, {
    method: 'POST'
  });
}

export async function selectDirectories(): Promise<string[] | null> {
  if (!isTauriEnvironment()) {
    return null;
  }
  try {
    const dialog = await import('@tauri-apps/api/dialog');
    const selection = await dialog.open({
      directory: true,
      multiple: true
    });
    if (selection === null) {
      return [];
    }
    return Array.isArray(selection) ? selection : [selection];
  } catch {
    return [];
  }
}
