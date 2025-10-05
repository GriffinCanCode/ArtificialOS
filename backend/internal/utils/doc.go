// Package utils provides utility functions for validation, hashing, and common operations.
//
// Utilities:
//   - Hash: Deterministic hashing for app identification
//   - Validation: Input validation for all data types
//   - AppIdentifier: Generate and verify app hashes
//
// Hashing:
//   - SHA256-based deterministic hashing
//   - App hash generation from title, parent, metadata
//   - Short hash generation for display
//   - Hash verification
//
// Validation:
//   - String length and format validation
//   - JSON size and depth validation
//   - Email, username, password validation
//   - ID, category, tag validation
//   - UI spec and context validation
//
// Features:
//   - Consistent error messages
//   - Configurable limits
//   - Type-safe validation functions
//
// Example Usage:
//
//	hasher := utils.NewHasher(utils.AlgorithmSHA256)
//	hash := hasher.HashString("calculator")
//
//	validator := utils.NewJSONSizeValidator(1024 * 1024)
//	err := validator.ValidateJSON(jsonData)
package utils
