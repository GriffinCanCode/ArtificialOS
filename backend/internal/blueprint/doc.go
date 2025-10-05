// Package blueprint provides parsing and transformation of Blueprint DSL specifications.
//
// Blueprint is a declarative language for defining AI-powered applications. This
// package handles parsing .bp files and converting them into executable app packages.
//
// Key Components:
//   - Parser: Blueprint DSL to Package transformation
//   - Template expansion and component resolution
//   - Service dependency extraction
//   - Metadata validation
//
// Blueprint Structure:
//   - app: Application metadata (id, name, permissions)
//   - services: Required service capabilities
//   - ui: Component tree and layout
//   - templates: Reusable component definitions
//   - config: Application configuration
//
// Example:
//
//	parser := blueprint.NewParser()
//	pkg, err := parser.Parse(blueprintContent)
package blueprint
