package utils

import (
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"sort"
	"strings"
)

// HashAlgorithm represents the hashing algorithm to use
type HashAlgorithm string

const (
	SHA256 HashAlgorithm = "sha256"
	// Extensible: add more algorithms here
	// SHA512 HashAlgorithm = "sha512"
	// BLAKE3 HashAlgorithm = "blake3"
)

// Hasher provides extensible hashing functionality
type Hasher struct {
	algorithm HashAlgorithm
}

// NewHasher creates a new hasher with the specified algorithm
func NewHasher(algorithm HashAlgorithm) *Hasher {
	return &Hasher{
		algorithm: algorithm,
	}
}

// DefaultHasher returns a hasher with the default algorithm
func DefaultHasher() *Hasher {
	return NewHasher(SHA256)
}

// Hash computes a hash of the input data
func (h *Hasher) Hash(data []byte) string {
	switch h.algorithm {
	case SHA256:
		hash := sha256.Sum256(data)
		return hex.EncodeToString(hash[:])
	// Extensible: add more cases here
	default:
		// Fallback to SHA256
		hash := sha256.Sum256(data)
		return hex.EncodeToString(hash[:])
	}
}

// HashString computes a hash of a string
func (h *Hasher) HashString(s string) string {
	return h.Hash([]byte(s))
}

// HashJSON computes a hash of a JSON-serializable object
// The hash is deterministic (same object = same hash)
func (h *Hasher) HashJSON(v interface{}) (string, error) {
	// Marshal to JSON with sorted keys for deterministic output
	data, err := json.Marshal(v)
	if err != nil {
		return "", fmt.Errorf("failed to marshal JSON: %w", err)
	}
	return h.Hash(data), nil
}

// HashFields computes a hash from multiple fields
// Fields are concatenated with a delimiter for consistent hashing
func (h *Hasher) HashFields(fields ...string) string {
	// Sort fields for deterministic ordering
	sorted := make([]string, len(fields))
	copy(sorted, fields)
	sort.Strings(sorted)

	combined := strings.Join(sorted, "|")
	return h.HashString(combined)
}

// AppIdentifier generates a unique identifier for an app based on its properties
type AppIdentifier struct {
	hasher *Hasher
}

// NewAppIdentifier creates a new app identifier
func NewAppIdentifier(hasher *Hasher) *AppIdentifier {
	if hasher == nil {
		hasher = DefaultHasher()
	}
	return &AppIdentifier{hasher: hasher}
}

// GenerateHash generates a deterministic hash for an app
// Uses title, creation context, and parent ID for uniqueness
func (ai *AppIdentifier) GenerateHash(title string, parentID *string, metadata map[string]interface{}) string {
	fields := []string{title}

	// Include parent ID if present
	if parentID != nil {
		fields = append(fields, fmt.Sprintf("parent:%s", *parentID))
	}

	// Include relevant metadata for uniqueness
	if request, ok := metadata["request"].(string); ok {
		fields = append(fields, fmt.Sprintf("request:%s", request))
	}

	return ai.hasher.HashFields(fields...)
}

// GenerateShortHash generates a short (8-character) hash for display
func (ai *AppIdentifier) GenerateShortHash(fullHash string) string {
	if len(fullHash) < 8 {
		return fullHash
	}
	return fullHash[:8]
}

// VerifyHash checks if a hash matches the expected app properties
func (ai *AppIdentifier) VerifyHash(hash string, title string, parentID *string, metadata map[string]interface{}) bool {
	expectedHash := ai.GenerateHash(title, parentID, metadata)
	return hash == expectedHash
}
