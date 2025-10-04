package scraper

import (
	"bytes"
	"fmt"
	"io"
	"regexp"
	"strings"
	"sync"

	"github.com/GriffinCanCode/AgentOS/backend/internal/types"
	"github.com/PuerkitoBio/goquery"
	"github.com/antchfx/htmlquery"
	"github.com/microcosm-cc/bluemonday"
	"github.com/saintfish/chardet"
	"golang.org/x/net/html"
	"golang.org/x/net/html/charset"
)

const (
	// MaxHTMLSize limits HTML input to 10MB to prevent memory exhaustion
	MaxHTMLSize = 10 * 1024 * 1024

	// DefaultTimeout for scraping operations
	DefaultTimeout = 30
)

// ScraperOps provides common scraping helpers
type ScraperOps struct {
	regexCache sync.Map
	sanitizer  *bluemonday.Policy
}

// NewScraperOps creates ops with sanitizer
func NewScraperOps() *ScraperOps {
	return &ScraperOps{
		sanitizer: bluemonday.UGCPolicy(),
	}
}

// Success creates successful result
func Success(data map[string]interface{}) (*types.Result, error) {
	return &types.Result{Success: true, Data: data}, nil
}

// Failure creates failed result
func Failure(message string) (*types.Result, error) {
	msg := message
	return &types.Result{Success: false, Error: &msg}, nil
}

// GetString extracts string from params with validation
func GetString(params map[string]interface{}, key string) (string, bool) {
	val, ok := params[key].(string)
	if !ok {
		return "", false
	}
	return val, true
}

// GetBool extracts bool from params with default
func GetBool(params map[string]interface{}, key string, defaultVal bool) bool {
	val, ok := params[key].(bool)
	if !ok {
		return defaultVal
	}
	return val
}

// GetInt extracts int from params with validation
func GetInt(params map[string]interface{}, key string) (int, bool) {
	switch v := params[key].(type) {
	case int:
		return v, true
	case float64:
		return int(v), true
	case int64:
		return int(v), true
	default:
		return 0, false
	}
}

// GetStringSlice extracts string slice from params
func GetStringSlice(params map[string]interface{}, key string) ([]string, bool) {
	val, ok := params[key].([]interface{})
	if !ok {
		return nil, false
	}

	result := make([]string, 0, len(val))
	for _, v := range val {
		if s, ok := v.(string); ok {
			result = append(result, s)
		}
	}
	return result, true
}

// ValidateHTML checks HTML size and returns error if too large
func ValidateHTML(html string) error {
	if len(html) == 0 {
		return fmt.Errorf("html content required")
	}
	if len(html) > MaxHTMLSize {
		return fmt.Errorf("html exceeds maximum size of %d bytes", MaxHTMLSize)
	}
	return nil
}

// DetectCharset detects and returns charset from HTML bytes
func DetectCharset(data []byte) string {
	detector := chardet.NewTextDetector()
	result, err := detector.DetectBest(data)
	if err != nil || result == nil {
		return "utf-8"
	}
	return strings.ToLower(result.Charset)
}

// LoadHTML loads HTML with automatic charset detection
func LoadHTML(htmlStr string) (*goquery.Document, error) {
	if err := ValidateHTML(htmlStr); err != nil {
		return nil, err
	}

	// Convert to bytes for charset detection
	data := []byte(htmlStr)

	// Detect charset
	detectedCharset := DetectCharset(data)

	// Parse with charset conversion
	reader := bytes.NewReader(data)
	utf8Reader, err := charset.NewReader(reader, detectedCharset)
	if err != nil {
		// Fallback to direct parsing
		return goquery.NewDocumentFromReader(strings.NewReader(htmlStr))
	}

	return goquery.NewDocumentFromReader(utf8Reader)
}

// LoadHTMLNode loads HTML into xpath-compatible node
func LoadHTMLNode(htmlStr string) (*html.Node, error) {
	if err := ValidateHTML(htmlStr); err != nil {
		return nil, err
	}

	data := []byte(htmlStr)
	detectedCharset := DetectCharset(data)

	reader := bytes.NewReader(data)
	utf8Reader, err := charset.NewReader(reader, detectedCharset)
	if err != nil {
		return htmlquery.Parse(strings.NewReader(htmlStr))
	}

	return htmlquery.Parse(utf8Reader)
}

// SanitizeHTML sanitizes HTML content
func (s *ScraperOps) SanitizeHTML(htmlStr string) string {
	return s.sanitizer.Sanitize(htmlStr)
}

// GetCachedRegex returns cached compiled regex
func (s *ScraperOps) GetCachedRegex(pattern string) (*regexp.Regexp, error) {
	// Check cache
	if cached, ok := s.regexCache.Load(pattern); ok {
		return cached.(*regexp.Regexp), nil
	}

	// Compile and cache
	re, err := regexp.Compile(pattern)
	if err != nil {
		return nil, err
	}

	s.regexCache.Store(pattern, re)
	return re, nil
}

// ExtractText safely extracts text from node
func ExtractText(n *html.Node) string {
	var buf bytes.Buffer
	var f func(*html.Node)
	f = func(n *html.Node) {
		if n.Type == html.TextNode {
			buf.WriteString(n.Data)
		}
		for c := n.FirstChild; c != nil; c = c.NextSibling {
			f(c)
		}
	}
	f(n)
	return strings.TrimSpace(buf.String())
}

// NormalizeWhitespace collapses multiple spaces into one
func NormalizeWhitespace(s string) string {
	return strings.Join(strings.Fields(s), " ")
}

// TruncateText truncates text to max length with ellipsis
func TruncateText(s string, maxLen int) string {
	if len(s) <= maxLen {
		return s
	}
	return s[:maxLen-3] + "..."
}

// LimitReader creates size-limited reader
func LimitReader(r io.Reader, maxBytes int64) io.Reader {
	return io.LimitReader(r, maxBytes)
}

// Deduplicate removes duplicate strings while preserving order
func Deduplicate(items []string) []string {
	seen := make(map[string]bool, len(items))
	result := make([]string, 0, len(items))

	for _, item := range items {
		if !seen[item] {
			seen[item] = true
			result = append(result, item)
		}
	}
	return result
}
