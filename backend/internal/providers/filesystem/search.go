package filesystem

import (
	"bufio"
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"strings"
	"sync"
	"time"

	"github.com/GriffinCanCode/AgentOS/backend/internal/types"
	"github.com/bmatcuk/doublestar/v4"
	"github.com/charlievieth/fastwalk"
)

// SearchOps handles search and filtering operations
type SearchOps struct {
	*FilesystemOps
}

// GetTools returns search operation tool definitions
func (s *SearchOps) GetTools() []types.Tool {
	return []types.Tool{
		{
			ID:          "filesystem.find",
			Name:        "Find Files",
			Description: "Find files by pattern (fast recursive)",
			Parameters: []types.Parameter{
				{Name: "path", Type: "string", Description: "Root directory", Required: true},
				{Name: "pattern", Type: "string", Description: "File pattern (e.g., '*.go')", Required: true},
			},
			Returns: "array",
		},
		{
			ID:          "filesystem.glob",
			Name:        "Advanced Glob",
			Description: "Advanced glob with ** patterns (gitignore-style)",
			Parameters: []types.Parameter{
				{Name: "path", Type: "string", Description: "Root directory", Required: true},
				{Name: "pattern", Type: "string", Description: "Glob pattern (e.g., '**/*.go')", Required: true},
			},
			Returns: "array",
		},
		{
			ID:          "filesystem.filter_by_extension",
			Name:        "Filter by Extension",
			Description: "Filter files by extensions (fast parallel)",
			Parameters: []types.Parameter{
				{Name: "path", Type: "string", Description: "Root directory", Required: true},
				{Name: "extensions", Type: "array", Description: "Extensions (e.g., ['.go', '.js'])", Required: true},
			},
			Returns: "array",
		},
		{
			ID:          "filesystem.filter_by_size",
			Name:        "Filter by Size",
			Description: "Filter files by size range",
			Parameters: []types.Parameter{
				{Name: "path", Type: "string", Description: "Root directory", Required: true},
				{Name: "min_size", Type: "number", Description: "Min size in bytes (0=no limit)", Required: false},
				{Name: "max_size", Type: "number", Description: "Max size in bytes (0=no limit)", Required: false},
			},
			Returns: "array",
		},
		{
			ID:          "filesystem.search_content",
			Name:        "Search Content",
			Description: "Search text in files (parallel worker pool)",
			Parameters: []types.Parameter{
				{Name: "path", Type: "string", Description: "Root directory", Required: true},
				{Name: "query", Type: "string", Description: "Text to search", Required: true},
				{Name: "extensions", Type: "array", Description: "File extensions to search", Required: false},
			},
			Returns: "array",
		},
		{
			ID:          "filesystem.regex_search",
			Name:        "Regex Search",
			Description: "Search files by regex pattern",
			Parameters: []types.Parameter{
				{Name: "path", Type: "string", Description: "Root directory", Required: true},
				{Name: "regex", Type: "string", Description: "Regex pattern", Required: true},
			},
			Returns: "array",
		},
		{
			ID:          "filesystem.filter_by_date",
			Name:        "Filter by Date",
			Description: "Filter files by modification date range",
			Parameters: []types.Parameter{
				{Name: "path", Type: "string", Description: "Root directory", Required: true},
				{Name: "after", Type: "string", Description: "After date (ISO 8601)", Required: false},
				{Name: "before", Type: "string", Description: "Before date (ISO 8601)", Required: false},
			},
			Returns: "array",
		},
		{
			ID:          "filesystem.recent_files",
			Name:        "Recent Files",
			Description: "Find recently modified files",
			Parameters: []types.Parameter{
				{Name: "path", Type: "string", Description: "Root directory", Required: true},
				{Name: "hours", Type: "number", Description: "Hours ago (default 24)", Required: false},
				{Name: "limit", Type: "number", Description: "Max results (default 50)", Required: false},
			},
			Returns: "array",
		},
	}
}

// Find finds files by pattern
func (s *SearchOps) Find(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return Failure("path parameter required")
	}
	pattern, ok := params["pattern"].(string)
	if !ok || pattern == "" {
		return Failure("pattern parameter required")
	}

	fullPath := s.resolvePath(ctx, path, appCtx)
	matches := []string{}
	conf := fastwalk.Config{Follow: false}

	err := fastwalk.Walk(&conf, fullPath, func(p string, d os.DirEntry, err error) error {
		// Check for context cancellation
		select {
		case <-ctx.Done():
			return ctx.Err()
		default:
		}

		if err != nil || d.IsDir() {
			return nil
		}

		matched, _ := filepath.Match(pattern, filepath.Base(p))
		if matched {
			relPath, _ := filepath.Rel(fullPath, p)
			matches = append(matches, relPath)
		}
		return nil
	})

	if err != nil {
		return Failure(fmt.Sprintf("find failed: %v", err))
	}

	return Success(map[string]interface{}{"path": path, "matches": matches, "count": len(matches)})
}

// Glob performs advanced glob matching
func (s *SearchOps) Glob(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return Failure("path parameter required")
	}
	pattern, ok := params["pattern"].(string)
	if !ok || pattern == "" {
		return Failure("pattern parameter required")
	}

	fullPath := s.resolvePath(ctx, path, appCtx)
	globPattern := filepath.Join(fullPath, pattern)

	matches, err := doublestar.FilepathGlob(globPattern)
	if err != nil {
		return Failure(fmt.Sprintf("glob failed: %v", err))
	}

	relMatches := []string{}
	for _, match := range matches {
		if relPath, err := filepath.Rel(fullPath, match); err == nil {
			relMatches = append(relMatches, relPath)
		}
	}

	return Success(map[string]interface{}{"path": path, "matches": relMatches, "count": len(relMatches)})
}

// FilterByExtension filters files by extensions
func (s *SearchOps) FilterByExtension(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return Failure("path parameter required")
	}

	extArr, ok := params["extensions"].([]interface{})
	if !ok || len(extArr) == 0 {
		return Failure("extensions array required")
	}

	extensions := make(map[string]bool)
	for _, ext := range extArr {
		if e, ok := ext.(string); ok {
			if !strings.HasPrefix(e, ".") {
				e = "." + e
			}
			extensions[e] = true
		}
	}

	fullPath := s.resolvePath(ctx, path, appCtx)
	matches := []string{}
	conf := fastwalk.Config{Follow: false}

	err := fastwalk.Walk(&conf, fullPath, func(p string, d os.DirEntry, err error) error {
		// Check for context cancellation
		select {
		case <-ctx.Done():
			return ctx.Err()
		default:
		}

		if err != nil || d.IsDir() {
			return nil
		}

		if extensions[filepath.Ext(p)] {
			relPath, _ := filepath.Rel(fullPath, p)
			matches = append(matches, relPath)
		}
		return nil
	})

	if err != nil {
		return Failure(fmt.Sprintf("filter failed: %v", err))
	}

	return Success(map[string]interface{}{"path": path, "matches": matches, "count": len(matches)})
}

// FilterBySize filters files by size range
func (s *SearchOps) FilterBySize(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return Failure("path parameter required")
	}

	minSize := int64(0)
	if min, ok := params["min_size"].(float64); ok {
		minSize = int64(min)
	}

	maxSize := int64(0)
	if max, ok := params["max_size"].(float64); ok {
		maxSize = int64(max)
	}

	fullPath := s.resolvePath(ctx, path, appCtx)
	matches := []map[string]interface{}{}
	conf := fastwalk.Config{Follow: false}

	err := fastwalk.Walk(&conf, fullPath, func(p string, d os.DirEntry, err error) error {
		// Check for context cancellation
		select {
		case <-ctx.Done():
			return ctx.Err()
		default:
		}

		if err != nil || d.IsDir() {
			return nil
		}

		info, err := d.Info()
		if err != nil {
			return nil
		}

		size := info.Size()
		if (minSize == 0 || size >= minSize) && (maxSize == 0 || size <= maxSize) {
			relPath, _ := filepath.Rel(fullPath, p)
			matches = append(matches, map[string]interface{}{
				"path": relPath,
				"size": size,
			})
		}
		return nil
	})

	if err != nil {
		return Failure(fmt.Sprintf("filter failed: %v", err))
	}

	return Success(map[string]interface{}{"path": path, "matches": matches, "count": len(matches)})
}

// SearchContent searches text in files
func (s *SearchOps) SearchContent(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return Failure("path parameter required")
	}
	query, ok := params["query"].(string)
	if !ok || query == "" {
		return Failure("query parameter required")
	}

	extensions := make(map[string]bool)
	if extArr, ok := params["extensions"].([]interface{}); ok {
		for _, ext := range extArr {
			if e, ok := ext.(string); ok {
				if !strings.HasPrefix(e, ".") {
					e = "." + e
				}
				extensions[e] = true
			}
		}
	}

	fullPath := s.resolvePath(ctx, path, appCtx)
	queryBytes := []byte(query)

	var mu sync.Mutex
	matches := []map[string]interface{}{}
	conf := fastwalk.Config{Follow: false}

	err := fastwalk.Walk(&conf, fullPath, func(p string, d os.DirEntry, err error) error {
		// Check for context cancellation
		select {
		case <-ctx.Done():
			return ctx.Err()
		default:
		}

		if err != nil || d.IsDir() {
			return nil
		}

		// Filter by extension if specified
		if len(extensions) > 0 && !extensions[filepath.Ext(p)] {
			return nil
		}

		file, err := os.Open(p)
		if err != nil {
			return nil
		}
		defer file.Close()

		scanner := bufio.NewScanner(file)
		lineNum := 1
		matchLines := []map[string]interface{}{}

		for scanner.Scan() {
			// Check for context cancellation during file scanning
			select {
			case <-ctx.Done():
				return ctx.Err()
			default:
			}

			if bytes.Contains(scanner.Bytes(), queryBytes) {
				matchLines = append(matchLines, map[string]interface{}{
					"line":    lineNum,
					"content": scanner.Text(),
				})
			}
			lineNum++
			if len(matchLines) > 100 { // Limit matches per file
				break
			}
		}

		if len(matchLines) > 0 {
			relPath, _ := filepath.Rel(fullPath, p)
			mu.Lock()
			matches = append(matches, map[string]interface{}{
				"path":    relPath,
				"matches": matchLines,
				"count":   len(matchLines),
			})
			mu.Unlock()
		}

		return nil
	})

	if err != nil {
		return Failure(fmt.Sprintf("search failed: %v", err))
	}

	return Success(map[string]interface{}{"path": path, "query": query, "results": matches, "files": len(matches)})
}

// RegexSearch searches files by regex
func (s *SearchOps) RegexSearch(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return Failure("path parameter required")
	}
	regexStr, ok := params["regex"].(string)
	if !ok || regexStr == "" {
		return Failure("regex parameter required")
	}

	regex, err := regexp.Compile(regexStr)
	if err != nil {
		return Failure(fmt.Sprintf("invalid regex: %v", err))
	}

	fullPath := s.resolvePath(ctx, path, appCtx)
	matches := []string{}
	conf := fastwalk.Config{Follow: false}

	err = fastwalk.Walk(&conf, fullPath, func(p string, d os.DirEntry, err error) error {
		// Check for context cancellation
		select {
		case <-ctx.Done():
			return ctx.Err()
		default:
		}

		if err != nil || d.IsDir() {
			return nil
		}

		if regex.MatchString(filepath.Base(p)) {
			relPath, _ := filepath.Rel(fullPath, p)
			matches = append(matches, relPath)
		}
		return nil
	})

	if err != nil {
		return Failure(fmt.Sprintf("search failed: %v", err))
	}

	return Success(map[string]interface{}{"path": path, "matches": matches, "count": len(matches)})
}

// FilterByDate filters files by date range
func (s *SearchOps) FilterByDate(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return Failure("path parameter required")
	}

	var after, before time.Time
	var err error

	if afterStr, ok := params["after"].(string); ok && afterStr != "" {
		after, err = time.Parse(time.RFC3339, afterStr)
		if err != nil {
			return Failure(fmt.Sprintf("invalid after date: %v", err))
		}
	}

	if beforeStr, ok := params["before"].(string); ok && beforeStr != "" {
		before, err = time.Parse(time.RFC3339, beforeStr)
		if err != nil {
			return Failure(fmt.Sprintf("invalid before date: %v", err))
		}
	}

	fullPath := s.resolvePath(ctx, path, appCtx)
	matches := []map[string]interface{}{}
	conf := fastwalk.Config{Follow: false}

	err = fastwalk.Walk(&conf, fullPath, func(p string, d os.DirEntry, err error) error {
		// Check for context cancellation
		select {
		case <-ctx.Done():
			return ctx.Err()
		default:
		}

		if err != nil || d.IsDir() {
			return nil
		}

		info, err := d.Info()
		if err != nil {
			return nil
		}

		modTime := info.ModTime()
		if (!after.IsZero() && modTime.Before(after)) || (!before.IsZero() && modTime.After(before)) {
			return nil
		}

		relPath, _ := filepath.Rel(fullPath, p)
		matches = append(matches, map[string]interface{}{
			"path":     relPath,
			"modified": modTime.Unix(),
		})
		return nil
	})

	if err != nil {
		return Failure(fmt.Sprintf("filter failed: %v", err))
	}

	return Success(map[string]interface{}{"path": path, "matches": matches, "count": len(matches)})
}

// RecentFiles finds recently modified files
func (s *SearchOps) RecentFiles(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	path, ok := params["path"].(string)
	if !ok || path == "" {
		return Failure("path parameter required")
	}

	hours := 24.0
	if h, ok := params["hours"].(float64); ok && h > 0 {
		hours = h
	}

	limit := 50
	if l, ok := params["limit"].(float64); ok && l > 0 {
		limit = int(l)
	}

	fullPath := s.resolvePath(ctx, path, appCtx)
	cutoff := time.Now().Add(-time.Duration(hours) * time.Hour)

	type fileInfo struct {
		path    string
		modTime time.Time
		size    int64
	}

	files := []fileInfo{}
	conf := fastwalk.Config{Follow: false}

	err := fastwalk.Walk(&conf, fullPath, func(p string, d os.DirEntry, err error) error {
		// Check for context cancellation
		select {
		case <-ctx.Done():
			return ctx.Err()
		default:
		}

		if err != nil || d.IsDir() {
			return nil
		}

		info, err := d.Info()
		if err != nil {
			return nil
		}

		if info.ModTime().After(cutoff) {
			relPath, _ := filepath.Rel(fullPath, p)
			files = append(files, fileInfo{relPath, info.ModTime(), info.Size()})
		}
		return nil
	})

	if err != nil {
		return Failure(fmt.Sprintf("search failed: %v", err))
	}

	// Sort by modification time (newest first)
	type fileSlice []fileInfo
	fs := fileSlice(files)
	for i := 0; i < len(fs); i++ {
		for j := i + 1; j < len(fs); j++ {
			if fs[j].modTime.After(fs[i].modTime) {
				fs[i], fs[j] = fs[j], fs[i]
			}
		}
	}

	// Limit results
	if len(files) > limit {
		files = files[:limit]
	}

	results := []map[string]interface{}{}
	for _, f := range files {
		results = append(results, map[string]interface{}{
			"path":     f.path,
			"modified": f.modTime.Unix(),
			"size":     f.size,
		})
	}

	return Success(map[string]interface{}{"path": path, "files": results, "count": len(results)})
}

// resolvePath resolves path through kernel
func (s *SearchOps) resolvePath(ctx context.Context, path string, appCtx *types.Context) string {
	pid := s.GetPID(appCtx)
	statData, err := s.Kernel.ExecuteSyscall(ctx, pid, "file_stat", map[string]interface{}{"path": path})
	if err != nil {
		return path
	}

	var statInfo map[string]interface{}
	if err := json.Unmarshal(statData, &statInfo); err != nil {
		return path
	}

	if fullPath, ok := statInfo["path"].(string); ok && fullPath != "" {
		return fullPath
	}
	return path
}
