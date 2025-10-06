// Package registry provides application package management for AI-OS.
//
// The registry stores reusable application packages that can be launched
// on demand. Packages include UI specifications, service dependencies,
// permissions, and metadata.
//
// Components:
//   - Manager: Package CRUD operations with caching
//   - Seeder: Loads .bp files from filesystem on startup
//
// Features:
//   - In-memory caching with size limits
//   - Automatic cache eviction (LRU)
//   - Persistent storage via kernel filesystem
//   - Category-based filtering
//   - Metadata-only listing for performance
//
// Storage Structure:
//   - Packages stored as JSON files
//   - Path: storage/registry/{package-id}.json
//   - Includes full blueprint and metadata
//
// Example Usage:
//
//	manager := registry.NewManager(kernel, storagePID, storagePath)
//	err := manager.Save(ctx, pkg)
//	pkg, err := manager.Load(ctx, "calculator")
//	packages, err := manager.List(&category)
package registry
