package providers

import (
	"context"
	"fmt"

	"github.com/GriffinCanCode/AgentOS/backend/internal/providers/filesystem"
	"github.com/GriffinCanCode/AgentOS/backend/internal/types"
)

// Filesystem provides comprehensive file system operations
type Filesystem struct {
	kernel      KernelClient
	storagePID  uint32
	storagePath string

	// Module instances
	basic      *filesystem.BasicOps
	directory  *filesystem.DirectoryOps
	operations *filesystem.OperationsOps
	metadata   *filesystem.MetadataOps
	search     *filesystem.SearchOps
	formats    *filesystem.FormatsOps
	archives   *filesystem.ArchivesOps
}

// NewFilesystem creates a modular filesystem provider
func NewFilesystem(kernel KernelClient, storagePID uint32, storagePath string) *Filesystem {
	ops := &filesystem.FilesystemOps{
		Kernel:      kernel,
		StoragePID:  storagePID,
		StoragePath: storagePath,
	}

	return &Filesystem{
		kernel:      kernel,
		storagePID:  storagePID,
		storagePath: storagePath,
		basic:       &filesystem.BasicOps{FilesystemOps: ops},
		directory:   &filesystem.DirectoryOps{FilesystemOps: ops},
		operations:  &filesystem.OperationsOps{FilesystemOps: ops},
		metadata:    &filesystem.MetadataOps{FilesystemOps: ops},
		search:      &filesystem.SearchOps{FilesystemOps: ops},
		formats:     &filesystem.FormatsOps{FilesystemOps: ops},
		archives:    &filesystem.ArchivesOps{FilesystemOps: ops},
	}
}

// Definition returns service metadata with all module tools
func (f *Filesystem) Definition() types.Service {
	// Collect tools from all modules
	tools := []types.Tool{}
	tools = append(tools, f.basic.GetTools()...)
	tools = append(tools, f.directory.GetTools()...)
	tools = append(tools, f.operations.GetTools()...)
	tools = append(tools, f.metadata.GetTools()...)
	tools = append(tools, f.search.GetTools()...)
	tools = append(tools, f.formats.GetTools()...)
	tools = append(tools, f.archives.GetTools()...)

	return types.Service{
		ID:          "filesystem",
		Name:        "Filesystem Service",
		Description: "Comprehensive file and directory operations with high-performance libraries",
		Category:    types.CategoryFilesystem,
		Capabilities: []string{
			"read", "write", "create", "delete",
			"directories", "search", "glob",
			"formats", "yaml", "json", "csv", "toml",
			"archives", "zip", "tar", "compression",
			"metadata", "mime", "timestamps",
		},
		Tools: tools,
	}
}

// Execute routes to appropriate module
func (f *Filesystem) Execute(ctx context.Context, toolID string, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	switch toolID {
	// Basic operations
	case "filesystem.read":
		return f.basic.Read(ctx, params, appCtx)
	case "filesystem.write":
		return f.basic.Write(ctx, params, appCtx)
	case "filesystem.append":
		return f.basic.Append(ctx, params, appCtx)
	case "filesystem.create":
		return f.basic.Create(ctx, params, appCtx)
	case "filesystem.delete":
		return f.basic.Delete(ctx, params, appCtx)
	case "filesystem.exists":
		return f.basic.Exists(ctx, params, appCtx)
	case "filesystem.read_lines":
		return f.basic.ReadLines(ctx, params, appCtx)
	case "filesystem.read_json":
		return f.basic.ReadJSON(ctx, params, appCtx)
	case "filesystem.write_json":
		return f.basic.WriteJSON(ctx, params, appCtx)
	case "filesystem.read_binary":
		return f.basic.ReadBinary(ctx, params, appCtx)
	case "filesystem.write_binary":
		return f.basic.WriteBinary(ctx, params, appCtx)
	case "filesystem.write_lines":
		return f.basic.WriteLines(ctx, params, appCtx)

	// Directory operations
	case "filesystem.dir.list", "filesystem.list":
		return f.directory.List(ctx, params, appCtx)
	case "filesystem.dir.create", "filesystem.mkdir":
		return f.directory.Create(ctx, params, appCtx)
	case "filesystem.dir.delete":
		return f.directory.Delete(ctx, params, appCtx)
	case "filesystem.dir.exists":
		return f.directory.Exists(ctx, params, appCtx)
	case "filesystem.dir.walk":
		return f.directory.Walk(ctx, params, appCtx)
	case "filesystem.dir.tree":
		return f.directory.Tree(ctx, params, appCtx)
	case "filesystem.dir.flatten":
		return f.directory.Flatten(ctx, params, appCtx)

	// File operations
	case "filesystem.copy":
		return f.operations.Copy(ctx, params, appCtx)
	case "filesystem.move":
		return f.operations.Move(ctx, params, appCtx)
	case "filesystem.rename":
		return f.operations.Rename(ctx, params, appCtx)
	case "filesystem.symlink":
		return f.operations.Symlink(ctx, params, appCtx)
	case "filesystem.readlink":
		return f.operations.Readlink(ctx, params, appCtx)
	case "filesystem.hardlink":
		return f.operations.Hardlink(ctx, params, appCtx)

	// Metadata operations
	case "filesystem.stat":
		return f.metadata.Stat(ctx, params, appCtx)
	case "filesystem.size":
		return f.metadata.Size(ctx, params, appCtx)
	case "filesystem.size_human":
		return f.metadata.SizeHuman(ctx, params, appCtx)
	case "filesystem.total_size":
		return f.metadata.TotalSize(ctx, params, appCtx)
	case "filesystem.modified_time":
		return f.metadata.ModifiedTime(ctx, params, appCtx)
	case "filesystem.created_time":
		return f.metadata.CreatedTime(ctx, params, appCtx)
	case "filesystem.accessed_time":
		return f.metadata.AccessedTime(ctx, params, appCtx)
	case "filesystem.mime_type":
		return f.metadata.MIMEType(ctx, params, appCtx)
	case "filesystem.is_text":
		return f.metadata.IsText(ctx, params, appCtx)
	case "filesystem.is_binary":
		return f.metadata.IsBinary(ctx, params, appCtx)

	// Search operations
	case "filesystem.find":
		return f.search.Find(ctx, params, appCtx)
	case "filesystem.glob":
		return f.search.Glob(ctx, params, appCtx)
	case "filesystem.filter_by_extension":
		return f.search.FilterByExtension(ctx, params, appCtx)
	case "filesystem.filter_by_size":
		return f.search.FilterBySize(ctx, params, appCtx)
	case "filesystem.search_content":
		return f.search.SearchContent(ctx, params, appCtx)
	case "filesystem.regex_search":
		return f.search.RegexSearch(ctx, params, appCtx)
	case "filesystem.filter_by_date":
		return f.search.FilterByDate(ctx, params, appCtx)
	case "filesystem.recent_files":
		return f.search.RecentFiles(ctx, params, appCtx)

	// Format operations
	case "filesystem.yaml.read":
		return f.formats.YAMLRead(ctx, params, appCtx)
	case "filesystem.yaml.write":
		return f.formats.YAMLWrite(ctx, params, appCtx)
	case "filesystem.csv.read":
		return f.formats.CSVRead(ctx, params, appCtx)
	case "filesystem.csv.write":
		return f.formats.CSVWrite(ctx, params, appCtx)
	case "filesystem.json.merge":
		return f.formats.JSONMerge(ctx, params, appCtx)
	case "filesystem.toml.read":
		return f.formats.TOMLRead(ctx, params, appCtx)
	case "filesystem.toml.write":
		return f.formats.TOMLWrite(ctx, params, appCtx)
	case "filesystem.csv.to_json":
		return f.formats.CSVToJSON(ctx, params, appCtx)

	// Archive operations
	case "filesystem.zip.create":
		return f.archives.ZIPCreate(ctx, params, appCtx)
	case "filesystem.zip.extract":
		return f.archives.ZIPExtract(ctx, params, appCtx)
	case "filesystem.zip.list":
		return f.archives.ZIPList(ctx, params, appCtx)
	case "filesystem.zip.add":
		return f.archives.ZIPAdd(ctx, params, appCtx)
	case "filesystem.tar.create":
		return f.archives.TARCreate(ctx, params, appCtx)
	case "filesystem.tar.extract":
		return f.archives.TARExtract(ctx, params, appCtx)
	case "filesystem.tar.list":
		return f.archives.TARList(ctx, params, appCtx)
	case "filesystem.extract_auto":
		return f.archives.ExtractAuto(ctx, params, appCtx)

	default:
		return failure(fmt.Sprintf("unknown tool: %s", toolID))
	}
}
