//go:build integration
// +build integration

package integration

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"testing"
	"time"

	"github.com/GriffinCanCode/AgentOS/backend/internal/providers"
	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
	"github.com/GriffinCanCode/AgentOS/backend/tests/helpers/testutil"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestHTTPRequestsIntegration(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping integration test in short mode")
	}

	// Create test server
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		switch r.URL.Path {
		case "/get":
			assert.Equal(t, "GET", r.Method)
			w.WriteHeader(http.StatusOK)
			json.NewEncoder(w).Encode(map[string]string{"message": "success"})

		case "/post":
			assert.Equal(t, "POST", r.Method)
			body, _ := io.ReadAll(r.Body)
			w.WriteHeader(http.StatusCreated)
			w.Write(body)

		case "/put":
			assert.Equal(t, "PUT", r.Method)
			w.WriteHeader(http.StatusOK)
			json.NewEncoder(w).Encode(map[string]string{"updated": "true"})

		case "/patch":
			assert.Equal(t, "PATCH", r.Method)
			w.WriteHeader(http.StatusOK)
			json.NewEncoder(w).Encode(map[string]string{"patched": "true"})

		case "/delete":
			assert.Equal(t, "DELETE", r.Method)
			w.WriteHeader(http.StatusNoContent)

		case "/head":
			assert.Equal(t, "HEAD", r.Method)
			w.Header().Set("Content-Type", "application/json")
			w.Header().Set("Content-Length", "100")
			w.WriteHeader(http.StatusOK)

		case "/options":
			assert.Equal(t, "OPTIONS", r.Method)
			w.Header().Set("Allow", "GET, POST, PUT, DELETE")
			w.WriteHeader(http.StatusOK)

		case "/auth":
			auth := r.Header.Get("Authorization")
			if auth == "" {
				w.WriteHeader(http.StatusUnauthorized)
				return
			}
			w.WriteHeader(http.StatusOK)
			json.NewEncoder(w).Encode(map[string]string{"authenticated": "true"})

		case "/slow":
			time.Sleep(100 * time.Millisecond)
			w.WriteHeader(http.StatusOK)
			json.NewEncoder(w).Encode(map[string]string{"slow": "true"})

		case "/redirect":
			http.Redirect(w, r, "/get", http.StatusFound)

		default:
			w.WriteHeader(http.StatusNotFound)
		}
	}))
	defer server.Close()

	httpProvider := providers.NewHTTP()
	ctx := context.Background()
	appCtx := &types.Context{}

	t.Run("GET request", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.get", map[string]interface{}{
			"url": server.URL + "/get",
		}, appCtx)

		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		assert.Equal(t, 200, result.Data["status"])
		assert.Contains(t, result.Data["body"], "success")
	})

	t.Run("GET with query params", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.get", map[string]interface{}{
			"url": server.URL + "/get",
			"params": map[string]interface{}{
				"page":  1,
				"limit": 10,
			},
		}, appCtx)

		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		assert.Equal(t, 200, result.Data["status"])
	})

	t.Run("POST request with JSON", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.post", map[string]interface{}{
			"url": server.URL + "/post",
			"data": map[string]interface{}{
				"name":  "test",
				"value": 123,
			},
			"json": true,
		}, appCtx)

		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		assert.Equal(t, 201, result.Data["status"])
	})

	t.Run("PUT request", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.put", map[string]interface{}{
			"url": server.URL + "/put",
			"data": map[string]interface{}{
				"id": 1,
			},
		}, appCtx)

		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		assert.Equal(t, 200, result.Data["status"])
	})

	t.Run("PATCH request", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.patch", map[string]interface{}{
			"url": server.URL + "/patch",
			"data": map[string]interface{}{
				"status": "updated",
			},
		}, appCtx)

		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		assert.Equal(t, 200, result.Data["status"])
	})

	t.Run("DELETE request", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.delete", map[string]interface{}{
			"url": server.URL + "/delete",
		}, appCtx)

		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		assert.Equal(t, 204, result.Data["status"])
	})

	t.Run("HEAD request", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.head", map[string]interface{}{
			"url": server.URL + "/head",
		}, appCtx)

		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		assert.Equal(t, 200, result.Data["status"])
		headers := result.Data["headers"].(map[string]string)
		assert.Equal(t, "application/json", headers["Content-Type"])
	})

	t.Run("OPTIONS request", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.options", map[string]interface{}{
			"url": server.URL + "/options",
		}, appCtx)

		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		assert.Equal(t, 200, result.Data["status"])
	})

	t.Run("custom headers", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.get", map[string]interface{}{
			"url": server.URL + "/get",
			"headers": map[string]interface{}{
				"X-Custom-Header": "test-value",
			},
		}, appCtx)

		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
	})
}

func TestHTTPConfigIntegration(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping integration test in short mode")
	}

	ctx := context.Background()
	appCtx := &types.Context{}

	t.Run("set and get headers", func(t *testing.T) {
		// Create a new provider for this test to avoid state pollution
		httpProvider := providers.NewHTTP()

		// Set header
		result, err := httpProvider.Execute(ctx, "http.setHeader", map[string]interface{}{
			"key":   "X-API-Key",
			"value": "secret-key",
		}, appCtx)
		require.NoError(t, err)
		testutil.AssertSuccess(t, result)

		// Get headers (using same provider instance)
		result, err = httpProvider.Execute(ctx, "http.getHeaders", map[string]interface{}{}, appCtx)
		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		headers := result.Data["headers"].(map[string]string)
		t.Logf("All headers: %+v", headers)
		assert.Contains(t, headers, "X-Api-Key", "X-API-Key should be in headers map")
		assert.Equal(t, "secret-key", headers["X-Api-Key"])

		// Remove header
		result, err = httpProvider.Execute(ctx, "http.removeHeader", map[string]interface{}{
			"key": "X-API-Key",
		}, appCtx)
		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
	})

	t.Run("set timeout", func(t *testing.T) {
		httpProvider := providers.NewHTTP()
		result, err := httpProvider.Execute(ctx, "http.setTimeout", map[string]interface{}{
			"seconds": 30.0,
		}, appCtx)

		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		testutil.AssertDataField(t, result, "seconds", 30.0)
	})

	t.Run("set bearer auth", func(t *testing.T) {
		httpProvider := providers.NewHTTP()
		result, err := httpProvider.Execute(ctx, "http.setAuth", map[string]interface{}{
			"type":  "bearer",
			"token": "test-token-123",
		}, appCtx)

		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		testutil.AssertDataField(t, result, "type", "bearer")

		// Clear auth
		result, err = httpProvider.Execute(ctx, "http.clearAuth", map[string]interface{}{}, appCtx)
		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
	})

	t.Run("set basic auth", func(t *testing.T) {
		httpProvider := providers.NewHTTP()
		result, err := httpProvider.Execute(ctx, "http.setAuth", map[string]interface{}{
			"type":     "basic",
			"username": "user",
			"password": "pass",
		}, appCtx)

		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		testutil.AssertDataField(t, result, "type", "basic")
	})

	t.Run("set custom auth", func(t *testing.T) {
		httpProvider := providers.NewHTTP()
		result, err := httpProvider.Execute(ctx, "http.setAuth", map[string]interface{}{
			"type":   "custom",
			"header": "Custom-Auth: value",
		}, appCtx)

		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		testutil.AssertDataField(t, result, "type", "custom")
	})

	t.Run("invalid auth type", func(t *testing.T) {
		httpProvider := providers.NewHTTP()
		result, err := httpProvider.Execute(ctx, "http.setAuth", map[string]interface{}{
			"type": "invalid",
		}, appCtx)

		require.NoError(t, err)
		testutil.AssertError(t, result)
	})
}

func TestHTTPResilienceIntegration(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping integration test in short mode")
	}

	httpProvider := providers.NewHTTP()
	ctx := context.Background()
	appCtx := &types.Context{}

	t.Run("set retry policy", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.setRetry", map[string]interface{}{
			"max_retries":      3.0,
			"min_wait_seconds": 1.0,
			"max_wait_seconds": 5.0,
		}, appCtx)

		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		assert.Equal(t, 3, result.Data["max_retries"])
		assert.Equal(t, 1.0, result.Data["min_wait_seconds"])
		assert.Equal(t, 5.0, result.Data["max_wait_seconds"])
	})

	t.Run("invalid retry config", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.setRetry", map[string]interface{}{
			"max_retries":      15.0, // Too high
			"min_wait_seconds": 1.0,
		}, appCtx)

		require.NoError(t, err)
		testutil.AssertError(t, result)
	})

	t.Run("set rate limit", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.setRateLimit", map[string]interface{}{
			"requests_per_second": 10.0,
		}, appCtx)

		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		testutil.AssertDataField(t, result, "requests_per_second", 10.0)
		assert.Equal(t, "token_bucket", result.Data["algorithm"])
	})

	t.Run("unlimited rate limit", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.setRateLimit", map[string]interface{}{
			"requests_per_second": 0.0,
		}, appCtx)

		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		assert.Equal(t, true, result.Data["unlimited"])
	})

	t.Run("get rate limit", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.getRateLimit", map[string]interface{}{}, appCtx)

		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		assert.Equal(t, true, result.Data["configured"])
	})
}

func TestHTTPConnectionIntegration(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping integration test in short mode")
	}

	httpProvider := providers.NewHTTP()
	ctx := context.Background()
	appCtx := &types.Context{}

	t.Run("set SSL verification", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.setVerifySSL", map[string]interface{}{
			"verify": false,
		}, appCtx)

		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		assert.Equal(t, false, result.Data["verify"])
		assert.Contains(t, result.Data, "warning")
	})

	t.Run("set follow redirects", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.setFollowRedirects", map[string]interface{}{
			"follow":        true,
			"max_redirects": 5.0,
		}, appCtx)

		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		assert.Equal(t, true, result.Data["follow"])
		assert.Equal(t, 5, result.Data["max_redirects"])
	})

	t.Run("disable redirects", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.setFollowRedirects", map[string]interface{}{
			"follow": false,
		}, appCtx)

		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		assert.Equal(t, false, result.Data["follow"])
	})

	t.Run("enable cookie jar", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.setCookieJar", map[string]interface{}{
			"enabled": true,
		}, appCtx)

		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		assert.Equal(t, true, result.Data["enabled"])
	})

	t.Run("set proxy", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.setProxy", map[string]interface{}{
			"proxy_url": "http://localhost:8080",
		}, appCtx)

		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		assert.Equal(t, "localhost:8080", result.Data["host"])

		// Remove proxy
		result, err = httpProvider.Execute(ctx, "http.removeProxy", map[string]interface{}{}, appCtx)
		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
	})

	t.Run("invalid proxy URL", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.setProxy", map[string]interface{}{
			"proxy_url": "ftp://invalid",
		}, appCtx)

		require.NoError(t, err)
		testutil.AssertError(t, result)
	})
}

func TestHTTPDownloadsIntegration(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping integration test in short mode")
	}

	// Create test server with downloadable content
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "text/plain")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte("test file content"))
	}))
	defer server.Close()

	httpProvider := providers.NewHTTP()
	ctx := context.Background()
	appCtx := &types.Context{}

	t.Run("download file", func(t *testing.T) {
		tmpDir := t.TempDir()
		downloadPath := filepath.Join(tmpDir, "downloaded.txt")

		result, err := httpProvider.Execute(ctx, "http.download", map[string]interface{}{
			"url":  server.URL,
			"path": downloadPath,
		}, appCtx)

		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		assert.Equal(t, true, result.Data["downloaded"])
		assert.Equal(t, downloadPath, result.Data["path"])

		// Verify file exists and has content
		content, err := os.ReadFile(downloadPath)
		require.NoError(t, err)
		assert.Equal(t, "test file content", string(content))
	})

	t.Run("download with custom filename", func(t *testing.T) {
		tmpDir := t.TempDir()

		result, err := httpProvider.Execute(ctx, "http.download", map[string]interface{}{
			"url":      server.URL,
			"path":     tmpDir,
			"filename": "custom.txt",
		}, appCtx)

		require.NoError(t, err)
		testutil.AssertSuccess(t, result)

		expectedPath := filepath.Join(tmpDir, "custom.txt")
		assert.Contains(t, result.Data["path"], "custom.txt")

		// Verify file exists
		_, err = os.Stat(expectedPath)
		require.NoError(t, err)
	})

	t.Run("download creates directories", func(t *testing.T) {
		tmpDir := t.TempDir()
		downloadPath := filepath.Join(tmpDir, "nested", "dir", "file.txt")

		result, err := httpProvider.Execute(ctx, "http.download", map[string]interface{}{
			"url":         server.URL,
			"path":        downloadPath,
			"create_dirs": true,
		}, appCtx)

		require.NoError(t, err)
		testutil.AssertSuccess(t, result)

		// Verify file exists in nested directory
		_, err = os.Stat(downloadPath)
		require.NoError(t, err)
	})
}

func TestHTTPUploadsIntegration(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping integration test in short mode")
	}

	// Create test server that accepts uploads
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "POST" {
			w.WriteHeader(http.StatusMethodNotAllowed)
			return
		}

		err := r.ParseMultipartForm(10 << 20) // 10 MB max
		if err != nil {
			w.WriteHeader(http.StatusBadRequest)
			return
		}

		w.WriteHeader(http.StatusOK)
		json.NewEncoder(w).Encode(map[string]string{
			"uploaded": "true",
			"files":    fmt.Sprint(len(r.MultipartForm.File)),
		})
	}))
	defer server.Close()

	httpProvider := providers.NewHTTP()
	ctx := context.Background()
	appCtx := &types.Context{}

	t.Run("upload single file", func(t *testing.T) {
		// Create temp file
		tmpFile := filepath.Join(t.TempDir(), "upload.txt")
		err := os.WriteFile(tmpFile, []byte("test content"), 0644)
		require.NoError(t, err)

		result, err := httpProvider.Execute(ctx, "http.uploadFile", map[string]interface{}{
			"url":       server.URL,
			"filepath":  tmpFile,
			"fieldname": "file",
		}, appCtx)

		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		assert.Equal(t, true, result.Data["uploaded"])
	})

	t.Run("upload with additional params", func(t *testing.T) {
		tmpFile := filepath.Join(t.TempDir(), "upload.txt")
		err := os.WriteFile(tmpFile, []byte("test content"), 0644)
		require.NoError(t, err)

		result, err := httpProvider.Execute(ctx, "http.uploadFile", map[string]interface{}{
			"url":      server.URL,
			"filepath": tmpFile,
			"params": map[string]interface{}{
				"title":       "Test File",
				"description": "A test upload",
			},
		}, appCtx)

		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
	})

	t.Run("upload file not found", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.uploadFile", map[string]interface{}{
			"url":      server.URL,
			"filepath": "/nonexistent/file.txt",
		}, appCtx)

		require.NoError(t, err)
		testutil.AssertError(t, result)
	})

	t.Run("upload multiple files", func(t *testing.T) {
		tmpDir := t.TempDir()

		// Create test files
		file1 := filepath.Join(tmpDir, "file1.txt")
		file2 := filepath.Join(tmpDir, "file2.txt")
		err := os.WriteFile(file1, []byte("content 1"), 0644)
		require.NoError(t, err)
		err = os.WriteFile(file2, []byte("content 2"), 0644)
		require.NoError(t, err)

		result, err := httpProvider.Execute(ctx, "http.uploadMultiple", map[string]interface{}{
			"url": server.URL,
			"files": []interface{}{
				map[string]interface{}{
					"path":      file1,
					"fieldname": "file1",
				},
				map[string]interface{}{
					"path":      file2,
					"fieldname": "file2",
				},
			},
		}, appCtx)

		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		assert.Equal(t, 2, result.Data["uploaded"])
	})
}

func TestHTTPParseIntegration(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping integration test in short mode")
	}

	httpProvider := providers.NewHTTP()
	ctx := context.Background()
	appCtx := &types.Context{}

	t.Run("parse JSON object", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.parseJSON", map[string]interface{}{
			"json": `{"name": "test", "value": 123}`,
		}, appCtx)

		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		assert.Equal(t, "object", result.Data["type"])
		data := result.Data["data"].(map[string]interface{})
		assert.Equal(t, "test", data["name"])
		assert.Equal(t, 123.0, data["value"])
	})

	t.Run("parse JSON array", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.parseJSON", map[string]interface{}{
			"json": `[1, 2, 3]`,
		}, appCtx)

		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		assert.Equal(t, "array", result.Data["type"])
	})

	t.Run("parse invalid JSON", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.parseJSON", map[string]interface{}{
			"json": `{invalid}`,
		}, appCtx)

		require.NoError(t, err)
		testutil.AssertError(t, result)
	})

	t.Run("stringify JSON", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.stringifyJSON", map[string]interface{}{
			"data": map[string]interface{}{
				"name":  "test",
				"value": 123,
			},
			"pretty": false,
		}, appCtx)

		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		jsonStr := result.Data["json"].(string)
		assert.Contains(t, jsonStr, "name")
		assert.Contains(t, jsonStr, "test")
	})

	t.Run("stringify JSON pretty", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.stringifyJSON", map[string]interface{}{
			"data": map[string]interface{}{
				"name": "test",
			},
			"pretty": true,
		}, appCtx)

		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		assert.Equal(t, true, result.Data["pretty"])
	})
}

func TestHTTPURLIntegration(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping integration test in short mode")
	}

	httpProvider := providers.NewHTTP()
	ctx := context.Background()
	appCtx := &types.Context{}

	t.Run("build URL", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.buildURL", map[string]interface{}{
			"base": "https://api.example.com",
			"path": "users/123",
			"params": map[string]interface{}{
				"page":  1,
				"limit": 10,
			},
		}, appCtx)

		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		url := result.Data["url"].(string)
		assert.Contains(t, url, "https://api.example.com")
		assert.Contains(t, url, "users/123")
		assert.Contains(t, url, "page=1")
		assert.Contains(t, url, "limit=10")
	})

	t.Run("parse URL", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.parseURL", map[string]interface{}{
			"url": "https://api.example.com:8080/users?page=1&limit=10#section",
		}, appCtx)

		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		assert.Equal(t, "https", result.Data["scheme"])
		assert.Equal(t, "api.example.com:8080", result.Data["host"])
		assert.Equal(t, "api.example.com", result.Data["hostname"])
		assert.Equal(t, "8080", result.Data["port"])
		assert.Equal(t, "/users", result.Data["path"])
		assert.Equal(t, "section", result.Data["fragment"])

		query := result.Data["query"].(map[string]interface{})
		assert.Equal(t, "1", query["page"])
		assert.Equal(t, "10", query["limit"])
	})

	t.Run("join path", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.joinPath", map[string]interface{}{
			"base": "https://api.example.com/v1",
			"segments": []interface{}{
				"users",
				"123",
				"posts",
			},
		}, appCtx)

		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		path := result.Data["path"].(string)
		assert.Contains(t, path, "users/123/posts")
	})

	t.Run("encode query", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.encodeQuery", map[string]interface{}{
			"params": map[string]interface{}{
				"name": "test user",
				"age":  25,
			},
		}, appCtx)

		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		query := result.Data["query"].(string)
		assert.Contains(t, query, "name=test+user")
		assert.Contains(t, query, "age=25")
	})

	t.Run("decode query", func(t *testing.T) {
		result, err := httpProvider.Execute(ctx, "http.decodeQuery", map[string]interface{}{
			"query": "name=test+user&age=25",
		}, appCtx)

		require.NoError(t, err)
		testutil.AssertSuccess(t, result)
		params := result.Data["params"].(map[string]interface{})
		assert.Equal(t, "test user", params["name"])
		assert.Equal(t, "25", params["age"])
	})
}
