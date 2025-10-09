/**
 * Search Hook
 * Intelligent file search with filters
 */

import { useState, useCallback } from 'react';
import type { NativeAppContext } from '../sdk';
import type { FileEntry, SearchFilters, UseSearchReturn, UseFileSystemReturn } from '../types';

export function useSearch(
  context: NativeAppContext,
  fs: UseFileSystemReturn
): UseSearchReturn {
  const { executor } = context;
  const [query, setQuery] = useState('');
  const [filters, setFilters] = useState<SearchFilters>({});
  const [results, setResults] = useState<FileEntry[]>([]);
  const [loading, setLoading] = useState(false);

  const execute = useCallback(async () => {
    if (!query.trim()) {
      setResults([]);
      return;
    }

    setLoading(true);

    try {
      // Build search based on filters
      const params: any = {
        path: fs.currentPath,
        query: query,
      };

      // Add extension filter if type is specified
      if (filters.type && filters.type !== 'all') {
        const extensionMap: Record<string, string[]> = {
          images: ['jpg', 'jpeg', 'png', 'gif', 'webp', 'bmp', 'svg'],
          documents: ['pdf', 'doc', 'docx', 'txt', 'rtf', 'odt'],
          code: ['js', 'ts', 'tsx', 'jsx', 'py', 'go', 'rs', 'java', 'c', 'cpp', 'html', 'css'],
        };

        if (extensionMap[filters.type]) {
          params.extensions = extensionMap[filters.type];
        }
      }

      // Use appropriate search based on filters
      let searchResults;

      if (filters.minSize !== undefined || filters.maxSize !== undefined) {
        // Size-based search
        searchResults = await executor.execute('filesystem.filter_by_size', {
          path: fs.currentPath,
          min_size: filters.minSize,
          max_size: filters.maxSize,
        });
      } else if (filters.modifiedAfter || filters.modifiedBefore) {
        // Date-based search
        searchResults = await executor.execute('filesystem.filter_by_date', {
          path: fs.currentPath,
          after: filters.modifiedAfter,
          before: filters.modifiedBefore,
        });
      } else {
        // Content search
        searchResults = await executor.execute('filesystem.search_content', params);
      }

      // Transform results to FileEntry format
      if (searchResults?.matches) {
        const entries: FileEntry[] = searchResults.matches.map((match: any) => ({
          name: match.path.split('/').pop() || match.path,
          path: match.path,
          size: match.size || 0,
          modified: match.modified || new Date().toISOString(),
          is_dir: false,
        }));
        setResults(entries);
      } else if (searchResults?.results) {
        const entries: FileEntry[] = searchResults.results.map((result: any) => ({
          name: result.path.split('/').pop() || result.path,
          path: result.path,
          size: 0,
          modified: new Date().toISOString(),
          is_dir: false,
        }));
        setResults(entries);
      } else {
        setResults([]);
      }
    } catch (err) {
      console.error('Search failed:', err);
      setResults([]);
    } finally {
      setLoading(false);
    }
  }, [query, filters, fs.currentPath, executor]);

  const clear = useCallback(() => {
    setQuery('');
    setResults([]);
    setFilters({});
  }, []);

  return {
    query,
    setQuery,
    filters,
    setFilters,
    results,
    loading,
    execute,
    clear,
  };
}

