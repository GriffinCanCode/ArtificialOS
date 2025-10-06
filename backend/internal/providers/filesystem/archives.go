package filesystem

import (
	"archive/tar"
	"archive/zip"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"strings"

	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
	"github.com/charlievieth/fastwalk"
	"github.com/klauspost/compress/gzip"
	"github.com/klauspost/compress/zstd"
)

// ArchivesOps handles archive operations (zip, tar, tar.gz, tar.zst)
type ArchivesOps struct {
	*FilesystemOps
}

// GetTools returns archive operation tool definitions
func (a *ArchivesOps) GetTools() []types.Tool {
	return []types.Tool{
		{
			ID:          "filesystem.zip.create",
			Name:        "Create ZIP",
			Description: "Create ZIP archive (fast compression)",
			Parameters: []types.Parameter{
				{Name: "source", Type: "string", Description: "Source directory", Required: true},
				{Name: "output", Type: "string", Description: "Output ZIP path", Required: true},
			},
			Returns: "object",
		},
		{
			ID:          "filesystem.zip.extract",
			Name:        "Extract ZIP",
			Description: "Extract ZIP archive (parallel)",
			Parameters: []types.Parameter{
				{Name: "archive", Type: "string", Description: "ZIP file path", Required: true},
				{Name: "destination", Type: "string", Description: "Destination directory", Required: true},
			},
			Returns: "object",
		},
		{
			ID:          "filesystem.zip.list",
			Name:        "List ZIP",
			Description: "List ZIP archive contents",
			Parameters: []types.Parameter{
				{Name: "archive", Type: "string", Description: "ZIP file path", Required: true},
			},
			Returns: "array",
		},
		{
			ID:          "filesystem.zip.add",
			Name:        "Add to ZIP",
			Description: "Add files to existing ZIP",
			Parameters: []types.Parameter{
				{Name: "archive", Type: "string", Description: "ZIP file path", Required: true},
				{Name: "files", Type: "array", Description: "Files to add", Required: true},
			},
			Returns: "boolean",
		},
		{
			ID:          "filesystem.tar.create",
			Name:        "Create TAR",
			Description: "Create TAR archive (gzip/zstd compression)",
			Parameters: []types.Parameter{
				{Name: "source", Type: "string", Description: "Source directory", Required: true},
				{Name: "output", Type: "string", Description: "Output TAR path", Required: true},
				{Name: "compression", Type: "string", Description: "Compression (none/gzip/zstd)", Required: false},
			},
			Returns: "object",
		},
		{
			ID:          "filesystem.tar.extract",
			Name:        "Extract TAR",
			Description: "Extract TAR archive (auto-detect compression)",
			Parameters: []types.Parameter{
				{Name: "archive", Type: "string", Description: "TAR file path", Required: true},
				{Name: "destination", Type: "string", Description: "Destination directory", Required: true},
			},
			Returns: "object",
		},
		{
			ID:          "filesystem.tar.list",
			Name:        "List TAR",
			Description: "List TAR archive contents",
			Parameters: []types.Parameter{
				{Name: "archive", Type: "string", Description: "TAR file path", Required: true},
			},
			Returns: "array",
		},
		{
			ID:          "filesystem.extract_auto",
			Name:        "Auto-Extract",
			Description: "Auto-detect and extract archive",
			Parameters: []types.Parameter{
				{Name: "archive", Type: "string", Description: "Archive file path", Required: true},
				{Name: "destination", Type: "string", Description: "Destination directory", Required: true},
			},
			Returns: "object",
		},
	}
}

// ZIPCreate creates a ZIP archive
func (a *ArchivesOps) ZIPCreate(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	source, ok := params["source"].(string)
	if !ok || source == "" {
		return Failure("source parameter required")
	}

	output, ok := params["output"].(string)
	if !ok || output == "" {
		return Failure("output parameter required")
	}

	fullSource := a.resolvePath(ctx, source, appCtx)
	fullOutput := a.resolvePath(ctx, filepath.Dir(output), appCtx)
	fullOutput = filepath.Join(fullOutput, filepath.Base(output))

	zipFile, err := os.Create(fullOutput)
	if err != nil {
		return Failure(fmt.Sprintf("create failed: %v", err))
	}
	defer zipFile.Close()

	zipWriter := zip.NewWriter(zipFile)
	defer zipWriter.Close()

	fileCount := 0
	totalSize := int64(0)
	conf := fastwalk.Config{Follow: false}

	err = fastwalk.Walk(&conf, fullSource, func(path string, d os.DirEntry, err error) error {
		// Check for context cancellation
		select {
		case <-ctx.Done():
			return ctx.Err()
		default:
		}

		if err != nil || path == fullSource {
			return nil
		}

		_, err = d.Info()
		if err != nil {
			return nil
		}

		relPath, _ := filepath.Rel(fullSource, path)

		if d.IsDir() {
			_, err := zipWriter.Create(relPath + "/")
			return err
		}

		writer, err := zipWriter.Create(relPath)
		if err != nil {
			return err
		}

		file, err := os.Open(path)
		if err != nil {
			return nil
		}
		defer file.Close()

		size, _ := io.Copy(writer, file)
		totalSize += size
		fileCount++

		return nil
	})

	if err != nil {
		return Failure(fmt.Sprintf("zip creation failed: %v", err))
	}

	return Success(map[string]interface{}{
		"created":    true,
		"output":     output,
		"files":      fileCount,
		"total_size": totalSize,
	})
}

// ZIPExtract extracts a ZIP archive
func (a *ArchivesOps) ZIPExtract(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	archive, ok := params["archive"].(string)
	if !ok || archive == "" {
		return Failure("archive parameter required")
	}

	destination, ok := params["destination"].(string)
	if !ok || destination == "" {
		return Failure("destination parameter required")
	}

	fullArchive := a.resolvePath(ctx, archive, appCtx)
	fullDest := a.resolvePath(ctx, destination, appCtx)

	reader, err := zip.OpenReader(fullArchive)
	if err != nil {
		return Failure(fmt.Sprintf("open failed: %v", err))
	}
	defer reader.Close()

	fileCount := 0

	for _, file := range reader.File {
		// Check for context cancellation
		select {
		case <-ctx.Done():
			return Failure(fmt.Sprintf("extraction cancelled: %v", ctx.Err()))
		default:
		}

		// Prevent zip-slip attacks
		destPath := filepath.Join(fullDest, file.Name)
		if !strings.HasPrefix(destPath, filepath.Clean(fullDest)+string(os.PathSeparator)) {
			continue
		}

		if file.FileInfo().IsDir() {
			os.MkdirAll(destPath, 0755)
			continue
		}

		if err := os.MkdirAll(filepath.Dir(destPath), 0755); err != nil {
			continue
		}

		srcFile, err := file.Open()
		if err != nil {
			continue
		}

		dstFile, err := os.Create(destPath)
		if err != nil {
			srcFile.Close()
			continue
		}

		_, err = io.Copy(dstFile, srcFile)
		srcFile.Close()
		dstFile.Close()

		if err == nil {
			fileCount++
		}
	}

	return Success(map[string]interface{}{
		"extracted":   true,
		"destination": destination,
		"files":       fileCount,
	})
}

// ZIPList lists ZIP contents
func (a *ArchivesOps) ZIPList(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	archive, ok := params["archive"].(string)
	if !ok || archive == "" {
		return Failure("archive parameter required")
	}

	fullArchive := a.resolvePath(ctx, archive, appCtx)

	reader, err := zip.OpenReader(fullArchive)
	if err != nil {
		return Failure(fmt.Sprintf("open failed: %v", err))
	}
	defer reader.Close()

	entries := []map[string]interface{}{}
	for _, file := range reader.File {
		info := file.FileInfo()
		entries = append(entries, map[string]interface{}{
			"name":              file.Name,
			"size":              info.Size(),
			"compressed_size":   file.CompressedSize64,
			"modified":          info.ModTime().Unix(),
			"is_dir":            info.IsDir(),
			"compression_ratio": float64(file.CompressedSize64) / float64(info.Size()+1) * 100,
		})
	}

	return Success(map[string]interface{}{"archive": archive, "entries": entries, "count": len(entries)})
}

// ZIPAdd adds files to ZIP (recreates archive)
func (a *ArchivesOps) ZIPAdd(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	archive, ok := params["archive"].(string)
	if !ok || archive == "" {
		return Failure("archive parameter required")
	}

	filesParam, ok := params["files"].([]interface{})
	if !ok || len(filesParam) == 0 {
		return Failure("files array required")
	}

	fullArchive := a.resolvePath(ctx, archive, appCtx)

	// Open existing ZIP
	reader, err := zip.OpenReader(fullArchive)
	if err != nil {
		return Failure(fmt.Sprintf("open failed: %v", err))
	}
	defer reader.Close()

	// Create temp file
	tempFile := fullArchive + ".tmp"
	outFile, err := os.Create(tempFile)
	if err != nil {
		return Failure(fmt.Sprintf("temp file failed: %v", err))
	}
	defer outFile.Close()

	writer := zip.NewWriter(outFile)
	defer writer.Close()

	// Copy existing entries
	for _, file := range reader.File {
		w, err := writer.Create(file.Name)
		if err != nil {
			continue
		}

		r, err := file.Open()
		if err != nil {
			continue
		}

		io.Copy(w, r)
		r.Close()
	}

	// Add new files
	for _, fileParam := range filesParam {
		filePath, ok := fileParam.(string)
		if !ok {
			continue
		}

		fullPath := a.resolvePath(ctx, filePath, appCtx)

		file, err := os.Open(fullPath)
		if err != nil {
			continue
		}

		w, err := writer.Create(filepath.Base(filePath))
		if err != nil {
			file.Close()
			continue
		}

		io.Copy(w, file)
		file.Close()
	}

	writer.Close()
	outFile.Close()
	reader.Close()

	// Replace original
	if err := os.Rename(tempFile, fullArchive); err != nil {
		return Failure(fmt.Sprintf("replace failed: %v", err))
	}

	return Success(map[string]interface{}{"added": true, "archive": archive})
}

// TARCreate creates a TAR archive
func (a *ArchivesOps) TARCreate(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	source, ok := params["source"].(string)
	if !ok || source == "" {
		return Failure("source parameter required")
	}

	output, ok := params["output"].(string)
	if !ok || output == "" {
		return Failure("output parameter required")
	}

	compression := "gzip"
	if comp, ok := params["compression"].(string); ok {
		compression = comp
	}

	fullSource := a.resolvePath(ctx, source, appCtx)
	fullOutput := a.resolvePath(ctx, filepath.Dir(output), appCtx)
	fullOutput = filepath.Join(fullOutput, filepath.Base(output))

	outFile, err := os.Create(fullOutput)
	if err != nil {
		return Failure(fmt.Sprintf("create failed: %v", err))
	}
	defer outFile.Close()

	var tarWriter *tar.Writer

	switch compression {
	case "gzip":
		gzWriter := gzip.NewWriter(outFile)
		defer gzWriter.Close()
		tarWriter = tar.NewWriter(gzWriter)
	case "zstd":
		zstdWriter, _ := zstd.NewWriter(outFile)
		defer zstdWriter.Close()
		tarWriter = tar.NewWriter(zstdWriter)
	default:
		tarWriter = tar.NewWriter(outFile)
	}
	defer tarWriter.Close()

	fileCount := 0
	totalSize := int64(0)
	conf := fastwalk.Config{Follow: false}

	err = fastwalk.Walk(&conf, fullSource, func(path string, d os.DirEntry, err error) error {
		// Check for context cancellation
		select {
		case <-ctx.Done():
			return ctx.Err()
		default:
		}

		if err != nil || path == fullSource {
			return nil
		}

		info, err := d.Info()
		if err != nil {
			return nil
		}

		relPath, _ := filepath.Rel(fullSource, path)

		header, err := tar.FileInfoHeader(info, "")
		if err != nil {
			return nil
		}
		header.Name = relPath

		if err := tarWriter.WriteHeader(header); err != nil {
			return err
		}

		if !d.IsDir() {
			file, err := os.Open(path)
			if err != nil {
				return nil
			}
			defer file.Close()

			size, _ := io.Copy(tarWriter, file)
			totalSize += size
			fileCount++
		}

		return nil
	})

	if err != nil {
		return Failure(fmt.Sprintf("tar creation failed: %v", err))
	}

	return Success(map[string]interface{}{
		"created":     true,
		"output":      output,
		"files":       fileCount,
		"total_size":  totalSize,
		"compression": compression,
	})
}

// TARExtract extracts a TAR archive
func (a *ArchivesOps) TARExtract(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	archive, ok := params["archive"].(string)
	if !ok || archive == "" {
		return Failure("archive parameter required")
	}

	destination, ok := params["destination"].(string)
	if !ok || destination == "" {
		return Failure("destination parameter required")
	}

	fullArchive := a.resolvePath(ctx, archive, appCtx)
	fullDest := a.resolvePath(ctx, destination, appCtx)

	file, err := os.Open(fullArchive)
	if err != nil {
		return Failure(fmt.Sprintf("open failed: %v", err))
	}
	defer file.Close()

	var tarReader *tar.Reader

	// Auto-detect compression
	if strings.HasSuffix(archive, ".gz") || strings.HasSuffix(archive, ".tgz") {
		gzReader, err := gzip.NewReader(file)
		if err != nil {
			return Failure(fmt.Sprintf("gzip failed: %v", err))
		}
		defer gzReader.Close()
		tarReader = tar.NewReader(gzReader)
	} else if strings.HasSuffix(archive, ".zst") {
		zstdReader, err := zstd.NewReader(file)
		if err != nil {
			return Failure(fmt.Sprintf("zstd failed: %v", err))
		}
		defer zstdReader.Close()
		tarReader = tar.NewReader(zstdReader)
	} else {
		tarReader = tar.NewReader(file)
	}

	fileCount := 0

	for {
		// Check for context cancellation
		select {
		case <-ctx.Done():
			return Failure(fmt.Sprintf("extraction cancelled: %v", ctx.Err()))
		default:
		}

		header, err := tarReader.Next()
		if err == io.EOF {
			break
		}
		if err != nil {
			continue
		}

		destPath := filepath.Join(fullDest, header.Name)
		if !strings.HasPrefix(destPath, filepath.Clean(fullDest)+string(os.PathSeparator)) {
			continue
		}

		switch header.Typeflag {
		case tar.TypeDir:
			os.MkdirAll(destPath, 0755)
		case tar.TypeReg:
			if err := os.MkdirAll(filepath.Dir(destPath), 0755); err != nil {
				continue
			}

			outFile, err := os.Create(destPath)
			if err != nil {
				continue
			}

			_, err = io.Copy(outFile, tarReader)
			outFile.Close()

			if err == nil {
				fileCount++
			}
		}
	}

	return Success(map[string]interface{}{
		"extracted":   true,
		"destination": destination,
		"files":       fileCount,
	})
}

// TARList lists TAR contents
func (a *ArchivesOps) TARList(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	archive, ok := params["archive"].(string)
	if !ok || archive == "" {
		return Failure("archive parameter required")
	}

	fullArchive := a.resolvePath(ctx, archive, appCtx)

	file, err := os.Open(fullArchive)
	if err != nil {
		return Failure(fmt.Sprintf("open failed: %v", err))
	}
	defer file.Close()

	var tarReader *tar.Reader

	if strings.HasSuffix(archive, ".gz") || strings.HasSuffix(archive, ".tgz") {
		gzReader, err := gzip.NewReader(file)
		if err != nil {
			return Failure(fmt.Sprintf("gzip failed: %v", err))
		}
		defer gzReader.Close()
		tarReader = tar.NewReader(gzReader)
	} else if strings.HasSuffix(archive, ".zst") {
		zstdReader, err := zstd.NewReader(file)
		if err != nil {
			return Failure(fmt.Sprintf("zstd failed: %v", err))
		}
		defer zstdReader.Close()
		tarReader = tar.NewReader(zstdReader)
	} else {
		tarReader = tar.NewReader(file)
	}

	entries := []map[string]interface{}{}

	for {
		// Check for context cancellation
		select {
		case <-ctx.Done():
			return Failure(fmt.Sprintf("listing cancelled: %v", ctx.Err()))
		default:
		}

		header, err := tarReader.Next()
		if err == io.EOF {
			break
		}
		if err != nil {
			continue
		}

		entries = append(entries, map[string]interface{}{
			"name":     header.Name,
			"size":     header.Size,
			"modified": header.ModTime.Unix(),
			"is_dir":   header.Typeflag == tar.TypeDir,
			"type":     string(header.Typeflag),
		})
	}

	return Success(map[string]interface{}{"archive": archive, "entries": entries, "count": len(entries)})
}

// ExtractAuto auto-detects and extracts archive
func (a *ArchivesOps) ExtractAuto(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	archive, ok := params["archive"].(string)
	if !ok || archive == "" {
		return Failure("archive parameter required")
	}

	destination, ok := params["destination"].(string)
	if !ok || destination == "" {
		return Failure("destination parameter required")
	}

	ext := strings.ToLower(filepath.Ext(archive))

	switch ext {
	case ".zip":
		return a.ZIPExtract(ctx, params, appCtx)
	case ".tar", ".tgz", ".gz", ".zst":
		return a.TARExtract(ctx, params, appCtx)
	default:
		return Failure(fmt.Sprintf("unsupported archive format: %s", ext))
	}
}

// resolvePath resolves path through kernel
func (a *ArchivesOps) resolvePath(ctx context.Context, path string, appCtx *types.Context) string {
	pid := a.GetPID(appCtx)
	statData, err := a.Kernel.ExecuteSyscall(ctx, pid, "file_stat", map[string]interface{}{"path": path})
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
