package unit

import (
	"context"
	"testing"

	"github.com/GriffinCanCode/AgentOS/backend/internal/providers"
	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
	"github.com/GriffinCanCode/AgentOS/backend/tests/helpers/testutil"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestHTTPProviderDefinition(t *testing.T) {
	httpProvider := providers.NewHTTP()
	def := httpProvider.Definition()

	t.Run("service metadata", func(t *testing.T) {
		assert.Equal(t, "http", def.ID)
		assert.Equal(t, "HTTP Service", def.Name)
		assert.Equal(t, types.CategoryHTTP, def.Category)
		assert.NotEmpty(t, def.Description)
		assert.NotEmpty(t, def.Capabilities)
		assert.NotEmpty(t, def.Tools)
	})

	t.Run("tools count", func(t *testing.T) {
		// Verify all modules contribute tools
		// Requests: 7, Config: 6, Downloads: 1, Uploads: 2,
		// Parse: 3, URL: 5, Resilience: 3, Connection: 4
		// Total: 31 tools
		assert.GreaterOrEqual(t, len(def.Tools), 31)
	})

	t.Run("request tools exist", func(t *testing.T) {
		toolIDs := extractToolIDs(def.Tools)
		assert.Contains(t, toolIDs, "http.get")
		assert.Contains(t, toolIDs, "http.post")
		assert.Contains(t, toolIDs, "http.put")
		assert.Contains(t, toolIDs, "http.patch")
		assert.Contains(t, toolIDs, "http.delete")
		assert.Contains(t, toolIDs, "http.head")
		assert.Contains(t, toolIDs, "http.options")
	})

	t.Run("config tools exist", func(t *testing.T) {
		toolIDs := extractToolIDs(def.Tools)
		assert.Contains(t, toolIDs, "http.setHeader")
		assert.Contains(t, toolIDs, "http.removeHeader")
		assert.Contains(t, toolIDs, "http.getHeaders")
		assert.Contains(t, toolIDs, "http.setTimeout")
		assert.Contains(t, toolIDs, "http.setAuth")
		assert.Contains(t, toolIDs, "http.clearAuth")
	})

	t.Run("download tools exist", func(t *testing.T) {
		toolIDs := extractToolIDs(def.Tools)
		assert.Contains(t, toolIDs, "http.download")
	})

	t.Run("upload tools exist", func(t *testing.T) {
		toolIDs := extractToolIDs(def.Tools)
		assert.Contains(t, toolIDs, "http.uploadFile")
		assert.Contains(t, toolIDs, "http.uploadMultiple")
	})

	t.Run("parse tools exist", func(t *testing.T) {
		toolIDs := extractToolIDs(def.Tools)
		assert.Contains(t, toolIDs, "http.parseJSON")
		assert.Contains(t, toolIDs, "http.parseXML")
		assert.Contains(t, toolIDs, "http.stringifyJSON")
	})

	t.Run("url tools exist", func(t *testing.T) {
		toolIDs := extractToolIDs(def.Tools)
		assert.Contains(t, toolIDs, "http.buildURL")
		assert.Contains(t, toolIDs, "http.parseURL")
		assert.Contains(t, toolIDs, "http.joinPath")
		assert.Contains(t, toolIDs, "http.encodeQuery")
		assert.Contains(t, toolIDs, "http.decodeQuery")
	})

	t.Run("resilience tools exist", func(t *testing.T) {
		toolIDs := extractToolIDs(def.Tools)
		assert.Contains(t, toolIDs, "http.setRetry")
		assert.Contains(t, toolIDs, "http.setRateLimit")
		assert.Contains(t, toolIDs, "http.getRateLimit")
	})

	t.Run("connection tools exist", func(t *testing.T) {
		toolIDs := extractToolIDs(def.Tools)
		assert.Contains(t, toolIDs, "http.setProxy")
		assert.Contains(t, toolIDs, "http.removeProxy")
		assert.Contains(t, toolIDs, "http.setVerifySSL")
		assert.Contains(t, toolIDs, "http.setFollowRedirects")
		assert.Contains(t, toolIDs, "http.setCookieJar")
	})
}

func TestHTTPConfigOperations(t *testing.T) {
	httpProvider := providers.NewHTTP()
	ctx := context.Background()

	t.Run("set and remove header", func(t *testing.T) {
		// Set header
		result, err := httpProvider.Execute(ctx, "http.setHeader", map[string]interface{}{
			"key":   "X-API-Key",
			"value": "test-key",
		}, nil)
		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		testutil.AssertDataField(t, result, "set", true)

		// Get headers
		result, err = httpProvider.Execute(ctx, "http.getHeaders", map[string]interface{}{}, nil)
		require.NoError(t, err)
		testutil.AssertSuccess(t, result)

		// Remove header
		result, err = httpProvider.Execute(ctx, "http.removeHeader", map[string]interface{}{
			"key": "X-API-Key",
		}, nil)
		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
	})

	t.Run("set header missing key", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.setHeader", map[string]interface{}{
			"value": "test",
		}, nil)
		require.NoError(t, err)
		testutil.AssertError(t, result)
	})

	t.Run("set timeout valid", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.setTimeout", map[string]interface{}{
			"seconds": 30.0,
		}, nil)
		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		testutil.AssertDataField(t, result, "seconds", 30.0)
	})

	t.Run("set timeout invalid", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.setTimeout", map[string]interface{}{
			"seconds": -5.0,
		}, nil)
		require.NoError(t, err)
		testutil.AssertError(t, result)
	})

	t.Run("set auth bearer", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.setAuth", map[string]interface{}{
			"type":  "bearer",
			"token": "test-token",
		}, nil)
		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		testutil.AssertDataField(t, result, "type", "bearer")
	})

	t.Run("set auth basic", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.setAuth", map[string]interface{}{
			"type":     "basic",
			"username": "user",
			"password": "pass",
		}, nil)
		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		testutil.AssertDataField(t, result, "type", "basic")
	})

	t.Run("set auth custom", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.setAuth", map[string]interface{}{
			"type":   "custom",
			"header": "Custom-Auth: value",
		}, nil)
		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
	})

	t.Run("set auth invalid type", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.setAuth", map[string]interface{}{
			"type": "invalid",
		}, nil)
		require.NoError(t, err)
		testutil.AssertError(t, result)
	})

	t.Run("set auth basic missing username", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.setAuth", map[string]interface{}{
			"type":     "basic",
			"password": "pass",
		}, nil)
		require.NoError(t, err)
		testutil.AssertError(t, result)
	})

	t.Run("clear auth", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.clearAuth", map[string]interface{}{}, nil)
		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		testutil.AssertDataField(t, result, "cleared", true)
	})
}

func TestHTTPResilienceOperations(t *testing.T) {
	httpProvider := providers.NewHTTP()
	ctx := context.Background()

	t.Run("set retry valid", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.setRetry", map[string]interface{}{
			"max_retries":      3.0,
			"min_wait_seconds": 1.0,
			"max_wait_seconds": 10.0,
		}, nil)
		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		assert.Equal(t, 3, result.Data["max_retries"])
		assert.Equal(t, 1.0, result.Data["min_wait_seconds"])
		assert.Equal(t, 10.0, result.Data["max_wait_seconds"])
		assert.Equal(t, "exponential_backoff", result.Data["strategy"])
	})

	t.Run("set retry too many", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.setRetry", map[string]interface{}{
			"max_retries": 15.0,
		}, nil)
		require.NoError(t, err)
		testutil.AssertError(t, result)
	})

	t.Run("set retry negative", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.setRetry", map[string]interface{}{
			"max_retries": -1.0,
		}, nil)
		require.NoError(t, err)
		testutil.AssertError(t, result)
	})

	t.Run("set retry invalid wait times", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.setRetry", map[string]interface{}{
			"max_retries":      3.0,
			"min_wait_seconds": 10.0,
			"max_wait_seconds": 5.0,
		}, nil)
		require.NoError(t, err)
		testutil.AssertError(t, result)
	})

	t.Run("set rate limit valid", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.setRateLimit", map[string]interface{}{
			"requests_per_second": 10.0,
		}, nil)
		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		testutil.AssertDataField(t, result, "requests_per_second", 10.0)
		testutil.AssertDataField(t, result, "algorithm", "token_bucket")
		testutil.AssertDataField(t, result, "thread_safe", true)
	})

	t.Run("set rate limit unlimited", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.setRateLimit", map[string]interface{}{
			"requests_per_second": 0.0,
		}, nil)
		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		testutil.AssertDataField(t, result, "unlimited", true)
	})

	t.Run("set rate limit negative", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.setRateLimit", map[string]interface{}{
			"requests_per_second": -5.0,
		}, nil)
		require.NoError(t, err)
		testutil.AssertError(t, result)
	})

	t.Run("get rate limit", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.getRateLimit", map[string]interface{}{}, nil)
		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		testutil.AssertDataField(t, result, "configured", true)
	})
}

func TestHTTPConnectionOperations(t *testing.T) {
	httpProvider := providers.NewHTTP()
	ctx := context.Background()

	t.Run("set proxy valid", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.setProxy", map[string]interface{}{
			"proxy_url": "http://localhost:8080",
		}, nil)
		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		assert.Equal(t, "localhost:8080", result.Data["host"])
	})

	t.Run("set proxy invalid scheme", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.setProxy", map[string]interface{}{
			"proxy_url": "ftp://proxy.example.com",
		}, nil)
		require.NoError(t, err)
		testutil.AssertError(t, result)
	})

	t.Run("set proxy malformed", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.setProxy", map[string]interface{}{
			"proxy_url": "not a url",
		}, nil)
		require.NoError(t, err)
		testutil.AssertError(t, result)
	})

	t.Run("remove proxy", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.removeProxy", map[string]interface{}{}, nil)
		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		testutil.AssertDataField(t, result, "removed", true)
	})

	t.Run("set SSL verification enabled", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.setVerifySSL", map[string]interface{}{
			"verify": true,
		}, nil)
		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		testutil.AssertDataField(t, result, "verify", true)
		assert.NotContains(t, result.Data, "warning")
	})

	t.Run("set SSL verification disabled", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.setVerifySSL", map[string]interface{}{
			"verify": false,
		}, nil)
		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		testutil.AssertDataField(t, result, "verify", false)
		assert.Contains(t, result.Data, "warning")
	})

	t.Run("set follow redirects enabled", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.setFollowRedirects", map[string]interface{}{
			"follow":        true,
			"max_redirects": 5.0,
		}, nil)
		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		testutil.AssertDataField(t, result, "follow", true)
		assert.Equal(t, 5, result.Data["max_redirects"])
	})

	t.Run("set follow redirects disabled", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.setFollowRedirects", map[string]interface{}{
			"follow": false,
		}, nil)
		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		testutil.AssertDataField(t, result, "follow", false)
	})

	t.Run("set cookie jar enabled", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.setCookieJar", map[string]interface{}{
			"enabled": true,
		}, nil)
		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		testutil.AssertDataField(t, result, "enabled", true)
	})

	t.Run("set cookie jar disabled", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.setCookieJar", map[string]interface{}{
			"enabled": false,
		}, nil)
		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		testutil.AssertDataField(t, result, "enabled", false)
	})
}

func TestHTTPParseOperations(t *testing.T) {
	httpProvider := providers.NewHTTP()
	ctx := context.Background()

	t.Run("parse JSON object", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.parseJSON", map[string]interface{}{
			"json": `{"name": "test", "value": 123, "active": true}`,
		}, nil)
		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		assert.Equal(t, "object", result.Data["type"])
		data := result.Data["data"].(map[string]interface{})
		assert.Equal(t, "test", data["name"])
		assert.Equal(t, 123.0, data["value"])
		assert.Equal(t, true, data["active"])
	})

	t.Run("parse JSON array", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.parseJSON", map[string]interface{}{
			"json": `[1, 2, 3, 4, 5]`,
		}, nil)
		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		assert.Equal(t, "array", result.Data["type"])
	})

	t.Run("parse JSON string", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.parseJSON", map[string]interface{}{
			"json": `"hello world"`,
		}, nil)
		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		assert.Equal(t, "string", result.Data["type"])
	})

	t.Run("parse JSON number", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.parseJSON", map[string]interface{}{
			"json": `42.5`,
		}, nil)
		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		assert.Equal(t, "number", result.Data["type"])
	})

	t.Run("parse JSON boolean", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.parseJSON", map[string]interface{}{
			"json": `true`,
		}, nil)
		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		assert.Equal(t, "boolean", result.Data["type"])
	})

	t.Run("parse JSON null", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.parseJSON", map[string]interface{}{
			"json": `null`,
		}, nil)
		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		assert.Equal(t, "null", result.Data["type"])
	})

	t.Run("parse JSON invalid", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.parseJSON", map[string]interface{}{
			"json": `{invalid json}`,
		}, nil)
		require.NoError(t, err)
		testutil.AssertError(t, result)
	})

	t.Run("parse JSON empty", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.parseJSON", map[string]interface{}{
			"json": `   `,
		}, nil)
		require.NoError(t, err)
		testutil.AssertError(t, result)
	})

	t.Run("stringify JSON compact", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.stringifyJSON", map[string]interface{}{
			"data": map[string]interface{}{
				"name":  "test",
				"value": 123,
			},
			"pretty": false,
		}, nil)
		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		jsonStr := result.Data["json"].(string)
		assert.Contains(t, jsonStr, "name")
		assert.Contains(t, jsonStr, "test")
		assert.NotContains(t, jsonStr, "\n")
	})

	t.Run("stringify JSON pretty", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.stringifyJSON", map[string]interface{}{
			"data": map[string]interface{}{
				"name": "test",
			},
			"pretty": true,
		}, nil)
		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		testutil.AssertDataField(t, result, "pretty", true)
	})

	t.Run("stringify JSON missing data", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.stringifyJSON", map[string]interface{}{}, nil)
		require.NoError(t, err)
		testutil.AssertError(t, result)
	})
}

func TestHTTPURLOperations(t *testing.T) {
	httpProvider := providers.NewHTTP()
	ctx := context.Background()

	t.Run("build URL simple", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.buildURL", map[string]interface{}{
			"base": "https://api.example.com",
			"path": "users/123",
		}, nil)
		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		url := result.Data["url"].(string)
		assert.Contains(t, url, "https://api.example.com")
		assert.Contains(t, url, "users/123")
	})

	t.Run("build URL with params", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.buildURL", map[string]interface{}{
			"base": "https://api.example.com/search",
			"params": map[string]interface{}{
				"q":     "test",
				"page":  1,
				"limit": 10,
			},
		}, nil)
		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		url := result.Data["url"].(string)
		assert.Contains(t, url, "q=test")
		assert.Contains(t, url, "page=1")
		assert.Contains(t, url, "limit=10")
	})

	t.Run("build URL with fragment", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.buildURL", map[string]interface{}{
			"base":     "https://example.com/page",
			"fragment": "section1",
		}, nil)
		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		url := result.Data["url"].(string)
		assert.Contains(t, url, "#section1")
	})

	t.Run("build URL invalid base", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.buildURL", map[string]interface{}{
			"base": "://invalid",
		}, nil)
		require.NoError(t, err)
		testutil.AssertError(t, result)
	})

	t.Run("parse URL complete", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.parseURL", map[string]interface{}{
			"url": "https://api.example.com:8080/users?page=1#top",
		}, nil)
		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		assert.Equal(t, "https", result.Data["scheme"])
		assert.Equal(t, "api.example.com:8080", result.Data["host"])
		assert.Equal(t, "api.example.com", result.Data["hostname"])
		assert.Equal(t, "8080", result.Data["port"])
		assert.Equal(t, "/users", result.Data["path"])
		assert.Equal(t, "top", result.Data["fragment"])
	})

	t.Run("parse URL invalid", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.parseURL", map[string]interface{}{
			"url": "://invalid url",
		}, nil)
		require.NoError(t, err)
		testutil.AssertError(t, result)
	})

	t.Run("join path", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.joinPath", map[string]interface{}{
			"base": "https://api.example.com/v1",
			"segments": []interface{}{
				"users",
				"123",
				"posts",
			},
		}, nil)
		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		path := result.Data["path"].(string)
		assert.Contains(t, path, "v1/users/123/posts")
	})

	t.Run("join path missing segments", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.joinPath", map[string]interface{}{
			"base": "https://api.example.com",
		}, nil)
		require.NoError(t, err)
		testutil.AssertError(t, result)
	})

	t.Run("encode query", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.encodeQuery", map[string]interface{}{
			"params": map[string]interface{}{
				"name": "test user",
				"age":  25,
				"tags": []interface{}{"a", "b"},
			},
		}, nil)
		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		query := result.Data["query"].(string)
		assert.NotEmpty(t, query)
		assert.Greater(t, result.Data["length"].(int), 0)
	})

	t.Run("encode query missing params", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.encodeQuery", map[string]interface{}{}, nil)
		require.NoError(t, err)
		testutil.AssertError(t, result)
	})

	t.Run("decode query", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.decodeQuery", map[string]interface{}{
			"query": "name=test+user&age=25&active=true",
		}, nil)
		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		params := result.Data["params"].(map[string]interface{})
		assert.Equal(t, "test user", params["name"])
		assert.Equal(t, "25", params["age"])
		assert.Equal(t, "true", params["active"])
	})

	t.Run("decode query with leading question mark", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.decodeQuery", map[string]interface{}{
			"query": "?name=test",
		}, nil)
		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
	})

	t.Run("decode query invalid", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.decodeQuery", map[string]interface{}{
			"query": "%ZZ%ZZ",
		}, nil)
		require.NoError(t, err)
		testutil.AssertError(t, result)
	})
}

func TestHTTPRequestParameterValidation(t *testing.T) {
	httpProvider := providers.NewHTTP()
	ctx := context.Background()

	t.Run("get missing url", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.get", map[string]interface{}{}, nil)
		require.NoError(t, err)
		testutil.AssertError(t, result)
	})

	t.Run("post missing data", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.post", map[string]interface{}{
			"url": "https://api.example.com",
		}, nil)
		require.NoError(t, err)
		testutil.AssertError(t, result)
	})

	t.Run("put missing data", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.put", map[string]interface{}{
			"url": "https://api.example.com",
		}, nil)
		require.NoError(t, err)
		testutil.AssertError(t, result)
	})
}

func TestHTTPUnknownTool(t *testing.T) {
	httpProvider := providers.NewHTTP()
	ctx := context.Background()

	result, err := httpProvider.Execute(ctx, "http.unknownTool", map[string]interface{}{}, nil)
	require.NoError(t, err)
	testutil.AssertError(t, result)
	assert.Contains(t, *result.Error, "unknown tool")
}

// Helper function to extract tool IDs from tool list
func extractToolIDs(tools []types.Tool) map[string]bool {
	ids := make(map[string]bool)
	for _, tool := range tools {
		ids[tool.ID] = true
	}
	return ids
}
