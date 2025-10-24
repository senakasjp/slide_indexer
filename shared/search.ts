import type { SlideIndexItem } from './types';

export interface ParsedSearchQuery {
  terms: string[];
  phrases: string[];
  wildcards: RegExp[];
  isEmpty: boolean;
}

export function parseSearchQuery(raw: string): ParsedSearchQuery {
  const terms: string[] = [];
  const phrases: string[] = [];
  const wildcards: RegExp[] = [];

  const tokenRegex = /"([^"]+)"|([^\s]+)/g;
  let match: RegExpExecArray | null;

  while ((match = tokenRegex.exec(raw)) !== null) {
    if (match[1]) {
      const phrase = match[1].trim().toLowerCase();
      if (phrase) {
        phrases.push(phrase);
      }
    } else if (match[2]) {
      const token = match[2].trim();
      if (!token) {
        continue;
      }
      if (token.includes('*') || token.includes('?')) {
        const pattern = token
          .replace(/[.+^${}()|[\]\\]/g, '\\$&')
          .replace(/\*/g, '.*')
          .replace(/\?/g, '.');
        try {
          wildcards.push(new RegExp(`.*${pattern}.*`, 'i'));
        } catch {
          // Invalid regex, skip
        }
      } else {
        terms.push(token.toLowerCase());
      }
    }
  }

  return {
    terms,
    phrases,
    wildcards,
    isEmpty: terms.length === 0 && phrases.length === 0 && wildcards.length === 0
  };
}

export function matchesSearchQuery(item: SlideIndexItem, query: ParsedSearchQuery): boolean {
  if (query.isEmpty) {
    return true;
  }

  const corpus = [
    item.name,
    item.path,
    item.snippet,
    ...item.slides.map((slide) => slide.text),
    item.keywords.join(' ')
  ]
    .join(' ')
    .toLowerCase();

  for (const phrase of query.phrases) {
    if (!corpus.includes(phrase)) {
      return false;
    }
  }

  for (const term of query.terms) {
    if (!corpus.includes(term)) {
      return false;
    }
  }

  for (const wildcard of query.wildcards) {
    if (!wildcard.test(corpus)) {
      return false;
    }
  }

  return true;
}


