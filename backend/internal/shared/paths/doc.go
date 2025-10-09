// Package paths provides standardized filesystem paths.
//
// This package defines the canonical directory structure for the entire system,
// mirroring the kernel's VFS path layout. All filesystem operations should use
// these constants to ensure consistency.
//
// # Directory Structure
//
//	/storage/
//	  ├── native-apps/   (prebuilt OS applications)
//	  ├── apps/          (user/AI-generated apps)
//	  ├── user/          (user files)
//	  │   ├── documents/
//	  │   ├── downloads/
//	  │   └── projects/
//	  ├── system/        (system config)
//	  └── lib/           (shared libraries)
//	/tmp/                (temporary files)
//	/cache/              (cache files)
//
// # Usage
//
//	import "github.com/GriffinCanCode/AgentOS/backend/internal/shared/paths"
//
//	// Get standard paths
//	userDocs := paths.Documents
//
//	// Get app-specific paths
//	app := paths.AppPath("my-app")
//	dataDir := app.DataDir()  // /storage/apps/my-app/data
//
//	// Validate paths
//	if paths.IsUserspacePath(somePath) {
//	    // Safe to access
//	}
package paths
