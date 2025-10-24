// Stub for Tauri-only build
// This function is only called in browser offline mode, which is not supported in Tauri
export async function extractPdfPagesWeb(_data: Uint8Array | ArrayBuffer): Promise<string[]> {
  throw new Error('Browser PDF extraction not available in Tauri mode');
}


