/*!
 * Search Syscalls Implementation
 * High-performance file and content search
 */

use crate::core::serialization::json;
use crate::core::types::Pid;
use crate::permissions::{PermissionChecker, PermissionRequest};
use crate::syscalls::core::executor::SyscallExecutorWithIpc;
use crate::syscalls::types::{SearchResult, SyscallResult};
use crate::vfs::traits::FileSystem;
use log::{trace, warn};
use std::path::{Path, PathBuf};

impl SyscallExecutorWithIpc {
    /// Search for files by name with fuzzy matching
    pub(in crate::syscalls) fn search_files(
        &self,
        pid: Pid,
        path: &PathBuf,
        query: &str,
        limit: usize,
        recursive: bool,
        case_sensitive: bool,
        threshold: f64,
    ) -> SyscallResult {
        trace!(
            "search_files: pid={}, path={}, query={}, limit={}, recursive={}, threshold={}",
            pid,
            path.display(),
            query,
            limit,
            recursive,
            threshold
        );

        // Check read permission
        let req = PermissionRequest::file_read(pid, path.clone());

        let response = self.permission_manager().check(&req);
        if !response.is_allowed() {
            warn!("search_files: permission denied");
            return SyscallResult::permission_denied("Permission denied for file search");
        }

        // Perform file search
        let results = match self.search_files_impl(path, query, limit, recursive, case_sensitive, threshold) {
            Ok(results) => results,
            Err(e) => {
                warn!("search_files: search failed: {}", e);
                return SyscallResult::error(format!("Search failed: {}", e));
            }
        };

        // Serialize results
        match json::to_vec(&results) {
            Ok(data) => SyscallResult::success_with_data(data),
            Err(e) => {
                warn!("search_files: serialization failed: {}", e);
                SyscallResult::error(format!("Serialization failed: {}", e))
            }
        }
    }

    /// Search file contents (grep-like)
    pub(in crate::syscalls) fn search_content(
        &self,
        pid: Pid,
        path: &PathBuf,
        query: &str,
        limit: usize,
        recursive: bool,
        case_sensitive: bool,
        include_path: bool,
    ) -> SyscallResult {
        trace!(
            "search_content: pid={}, path={}, query={}, limit={}, recursive={}",
            pid,
            path.display(),
            query,
            limit,
            recursive
        );

        // Check read permission
        let req = PermissionRequest::file_read(pid, path.clone());

        let response = self.permission_manager().check(&req);
        if !response.is_allowed() {
            warn!("search_content: permission denied");
            return SyscallResult::permission_denied("Permission denied for content search");
        }

        // Perform content search
        let results = match self.search_content_impl(
            path,
            query,
            limit,
            recursive,
            case_sensitive,
            include_path,
        ) {
            Ok(results) => results,
            Err(e) => {
                warn!("search_content: search failed: {}", e);
                return SyscallResult::error(format!("Search failed: {}", e));
            }
        };

        // Serialize results
        match json::to_vec(&results) {
            Ok(data) => SyscallResult::success_with_data(data),
            Err(e) => {
                warn!("search_content: serialization failed: {}", e);
                SyscallResult::error(format!("Serialization failed: {}", e))
            }
        }
    }

    /// Implementation of file search with fuzzy matching
    fn search_files_impl(
        &self,
        path: &Path,
        query: &str,
        limit: usize,
        recursive: bool,
        case_sensitive: bool,
        threshold: f64,
    ) -> Result<Vec<SearchResult>, String> {
        use crate::core::memory::arena::with_arena;

        with_arena(|arena| {
            let mut results = bumpalo::collections::Vec::new_in(arena);
            let query_lower = if case_sensitive {
                query.to_string()
            } else {
                query.to_lowercase()
            };

            self.search_files_recursive(
                path,
                &query_lower,
                &mut results,
                limit,
                recursive,
                case_sensitive,
                threshold,
            )?;

            // Sort by score (best matches first)
            results.sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal));

            // Convert to owned Vec
            Ok(results.iter().cloned().collect())
        })
    }

    /// Recursive file search helper
    fn search_files_recursive(
        &self,
        path: &Path,
        query: &str,
        results: &mut bumpalo::collections::Vec<SearchResult>,
        limit: usize,
        recursive: bool,
        case_sensitive: bool,
        threshold: f64,
    ) -> Result<(), String> {
        if results.len() >= limit {
            return Ok(());
        }

        // Get VFS or return error
        let vfs = self.optional().vfs.as_ref().ok_or_else(|| "VFS not available".to_string())?;

        // List directory
        let entries = vfs.list_dir(path).map_err(|e| e.to_string())?;

        for entry in entries {
            if results.len() >= limit {
                break;
            }

            let entry_path = path.join(&entry.name);
            let name = if case_sensitive {
                entry.name.to_string()
            } else {
                entry.name.to_lowercase()
            };

            // Calculate fuzzy match score
            let score = self.fuzzy_match_score(&name, query);

            // Add to results if score is below threshold
            if score <= threshold {
                results.push(SearchResult {
                    path: entry_path.clone(),
                    score,
                    content: None,
                    line_number: None,
                });
            }

            // Recurse into directories if enabled
            if recursive && entry.file_type == crate::vfs::FileType::Directory {
                self.search_files_recursive(
                    &entry_path,
                    query,
                    results,
                    limit,
                    recursive,
                    case_sensitive,
                    threshold,
                )?;
            }
        }

        Ok(())
    }

    /// Implementation of content search
    fn search_content_impl(
        &self,
        path: &Path,
        query: &str,
        limit: usize,
        recursive: bool,
        case_sensitive: bool,
        include_path: bool,
    ) -> Result<Vec<SearchResult>, String> {
        use crate::core::memory::arena::with_arena;

        with_arena(|arena| {
            let mut results = bumpalo::collections::Vec::new_in(arena);
            let query_lower = if case_sensitive {
                query.to_string()
            } else {
                query.to_lowercase()
            };

            self.search_content_recursive(
                path,
                &query_lower,
                &mut results,
                limit,
                recursive,
                case_sensitive,
                include_path,
            )?;

            // Convert to owned Vec
            Ok(results.iter().cloned().collect())
        })
    }

    /// Recursive content search helper
    fn search_content_recursive(
        &self,
        path: &Path,
        query: &str,
        results: &mut bumpalo::collections::Vec<SearchResult>,
        limit: usize,
        recursive: bool,
        case_sensitive: bool,
        include_path: bool,
    ) -> Result<(), String> {
        if results.len() >= limit {
            return Ok(());
        }

        // Get VFS or return error
        let vfs = self.optional().vfs.as_ref().ok_or_else(|| "VFS not available".to_string())?;

        // List directory
        let entries = vfs.list_dir(path).map_err(|e| e.to_string())?;

        for entry in entries {
            if results.len() >= limit {
                break;
            }

            let entry_path = path.join(&entry.name);

            match entry.file_type {
                crate::vfs::FileType::File => {
                    // Read file and search content
                    if let Ok(content) = vfs.read(&entry_path) {
                        if let Ok(text) = String::from_utf8(content) {
                            let search_text = if case_sensitive {
                                text.clone()
                            } else {
                                text.to_lowercase()
                            };

                            // Search each line
                            for (line_num, line) in search_text.lines().enumerate() {
                                if results.len() >= limit {
                                    break;
                                }

                                if line.contains(query) {
                                    let content_str = if include_path {
                                        format!("{}:{}: {}", entry_path.display(), line_num + 1, text.lines().nth(line_num).unwrap_or(""))
                                    } else {
                                        text.lines().nth(line_num).unwrap_or("").to_string()
                                    };

                                    results.push(SearchResult {
                                        path: entry_path.clone(),
                                        score: 0.0, // Exact match
                                        content: Some(content_str),
                                        line_number: Some(line_num + 1),
                                    });
                                }
                            }
                        }
                    }
                }
                crate::vfs::FileType::Directory if recursive => {
                    self.search_content_recursive(
                        &entry_path,
                        query,
                        results,
                        limit,
                        recursive,
                        case_sensitive,
                        include_path,
                    )?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Fuzzy string matching using Levenshtein distance
    /// Returns normalized score (0.0 = perfect match, 1.0 = worst)
    fn fuzzy_match_score(&self, text: &str, pattern: &str) -> f64 {
        if text == pattern {
            return 0.0;
        }

        // Check if pattern is substring (higher priority)
        if text.contains(pattern) {
            let pos = text.find(pattern).unwrap();
            // Score based on position (earlier is better)
            return 0.1 + (pos as f64 / text.len() as f64) * 0.2;
        }

        // Calculate Levenshtein distance
        let distance = self.levenshtein_distance(text, pattern);
        let max_len = text.len().max(pattern.len());

        if max_len == 0 {
            return 1.0;
        }

        // Normalize to 0-1 range
        distance as f64 / max_len as f64
    }

    /// Levenshtein distance algorithm
    fn levenshtein_distance(&self, s1: &str, s2: &str) -> usize {
        let len1 = s1.chars().count();
        let len2 = s2.chars().count();

        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }

        let s1_chars: Vec<char> = s1.chars().collect();
        let s2_chars: Vec<char> = s2.chars().collect();

        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if s1_chars[i - 1] == s2_chars[j - 1] { 0 } else { 1 };
                matrix[i][j] = (matrix[i - 1][j] + 1)
                    .min(matrix[i][j - 1] + 1)
                    .min(matrix[i - 1][j - 1] + cost);
            }
        }

        matrix[len1][len2]
    }
}
