// Package service provides the service registry for AI-OS provider management.
//
// The registry maintains a catalog of available service providers and handles
// service discovery, tool execution, and relevance scoring for AI queries.
//
// Components:
//   - Registry: Central service catalog
//   - Provider: Interface for service implementations
//   - Service discovery with relevance scoring
//
// Features:
//   - Thread-safe service registration
//   - Category-based filtering
//   - Intent-based discovery with scoring
//   - Tool execution with context passing
//   - Service statistics and health
//
// Discovery Algorithm:
//   - Keyword matching in name/description
//   - Capability matching
//   - Category bonus for exact matches
//   - Score-based ranking
//
// Example Usage:
//
//	registry := service.NewRegistry()
//	registry.Register(filesystemProvider)
//	services := registry.Discover("read file", 5)
//	result, err := registry.Execute(ctx, "fs.read", params, appCtx)
package service
