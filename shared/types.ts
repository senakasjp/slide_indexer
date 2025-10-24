export type SlideKind = 'pptx' | 'pdf' | 'ppt';

export interface SlidePreview {
  index: number;
  text: string;
}

export interface SlideIndexItem {
  id: string;
  path: string;
  name: string;
  kind: SlideKind;
  slideCount: number | null;
  snippet: string;
  keywords: string[];
  createdAt: number;
  updatedAt: number;
  slides: SlidePreview[];
  checksum?: string;
}

export interface AppState {
  directories: string[];
  items: SlideIndexItem[];
  lastIndexedAt: number | null;
  warnings: string[];
}

export interface ScanSummary {
  indexed: number;
  scanned?: number;
  cached?: number;
  errors: string[];
  lastIndexedAt: number | null;
}
