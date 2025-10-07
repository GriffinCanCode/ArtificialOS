package http

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"time"

	"github.com/GriffinCanCode/AgentOS/backend/internal/grpc/kernel"
	"github.com/GriffinCanCode/AgentOS/backend/internal/infrastructure/monitoring"
	"github.com/GriffinCanCode/AgentOS/backend/internal/infrastructure/resilience"
	"github.com/gin-gonic/gin"
)

// MetricsAggregator collects metrics from all services with circuit breaker protection
type MetricsAggregator struct {
	metrics    *monitoring.Metrics
	kernel     *kernel.KernelClient
	httpClient *http.Client
	breaker    *resilience.Breaker
}

// NewMetricsAggregator creates a metrics aggregator with circuit breaker
func NewMetricsAggregator(metrics *monitoring.Metrics, kernel *kernel.KernelClient) *MetricsAggregator {
	// Create persistent HTTP client for metrics collection
	httpClient := &http.Client{
		Timeout: 5 * time.Second,
		Transport: &http.Transport{
			MaxIdleConns:        10,
			MaxIdleConnsPerHost: 2,
			IdleConnTimeout:     30 * time.Second,
		},
	}

	// Create circuit breaker for metrics HTTP calls
	breaker := resilience.New("metrics-http", resilience.Settings{
		MaxRequests: 3,
		Interval:    60 * time.Second,
		Timeout:     10 * time.Second,
		ReadyToTrip: func(counts resilience.Counts) bool {
			// Trip after 3 consecutive failures (metrics are non-critical)
			return counts.ConsecutiveFailures >= 3
		},
	})

	return &MetricsAggregator{
		metrics:    metrics,
		kernel:     kernel,
		httpClient: httpClient,
		breaker:    breaker,
	}
}

// MetricsSnapshot represents a snapshot of all system metrics
type MetricsSnapshot struct {
	Timestamp time.Time              `json:"timestamp"`
	Backend   map[string]interface{} `json:"backend"`
	Kernel    map[string]interface{} `json:"kernel,omitempty"`
	AIService map[string]interface{} `json:"ai_service,omitempty"`
	Summary   MetricsSummary         `json:"summary"`
}

// MetricsSummary provides high-level metrics
type MetricsSummary struct {
	TotalRequests     int64   `json:"total_requests"`
	AverageLatencyMs  float64 `json:"average_latency_ms"`
	ErrorRate         float64 `json:"error_rate"`
	ActiveConnections int     `json:"active_connections"`
	UptimeSeconds     float64 `json:"uptime_seconds"`
}

// GetAggregatedMetrics returns all metrics from all services
func (ma *MetricsAggregator) GetAggregatedMetrics(c *gin.Context) {
	snapshot := MetricsSnapshot{
		Timestamp: time.Now(),
		Backend:   ma.getBackendMetrics(),
		Summary:   ma.calculateSummary(),
	}

	// Try to get kernel metrics if available
	if ma.kernel != nil {
		if kernelMetrics := ma.getKernelMetrics(); kernelMetrics != nil {
			snapshot.Kernel = kernelMetrics
		}
	}

	// Try to get AI service metrics if available
	if aiMetrics := ma.getAIMetrics(); aiMetrics != nil {
		snapshot.AIService = aiMetrics
	}

	c.JSON(http.StatusOK, snapshot)
}

// GetMetricsDashboard returns an HTML dashboard
func (ma *MetricsAggregator) GetMetricsDashboard(c *gin.Context) {
	html := `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>AgentOS Metrics Dashboard</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
            background: #0a0a0a;
            color: #e0e0e0;
            padding: 20px;
        }
        .container { max-width: 1400px; margin: 0 auto; }
        h1 {
            font-size: 2rem;
            margin-bottom: 10px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
        }
        .subtitle { color: #888; margin-bottom: 30px; }
        .grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 20px;
            margin-bottom: 20px;
        }
        .card {
            background: #1a1a1a;
            border-radius: 12px;
            padding: 20px;
            border: 1px solid #333;
            transition: transform 0.2s, border-color 0.2s;
        }
        .card:hover {
            transform: translateY(-2px);
            border-color: #667eea;
        }
        .card h2 {
            font-size: 1.2rem;
            margin-bottom: 15px;
            color: #667eea;
        }
        .metric {
            display: flex;
            justify-content: space-between;
            padding: 10px 0;
            border-bottom: 1px solid #2a2a2a;
        }
        .metric:last-child { border-bottom: none; }
        .metric-label { color: #999; }
        .metric-value {
            font-weight: 600;
            color: #fff;
            font-family: 'Courier New', monospace;
        }
        .metric-value.good { color: #4ade80; }
        .metric-value.warning { color: #fbbf24; }
        .metric-value.error { color: #f87171; }
        .refresh-btn {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            border: none;
            padding: 12px 24px;
            border-radius: 8px;
            cursor: pointer;
            font-size: 1rem;
            margin-bottom: 20px;
            transition: opacity 0.2s;
        }
        .refresh-btn:hover { opacity: 0.9; }
        .timestamp {
            color: #666;
            text-align: center;
            margin-top: 20px;
            font-size: 0.9rem;
        }
        .endpoint-link {
            display: inline-block;
            margin: 10px 10px 20px 0;
            padding: 8px 16px;
            background: #2a2a2a;
            color: #667eea;
            text-decoration: none;
            border-radius: 6px;
            font-size: 0.9rem;
            border: 1px solid #333;
            transition: all 0.2s;
        }
        .endpoint-link:hover {
            background: #333;
            border-color: #667eea;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>🚀 AgentOS Metrics Dashboard</h1>
        <p class="subtitle">Real-time performance monitoring across all services</p>

        <div>
            <a href="/metrics" class="endpoint-link">📊 Prometheus Metrics</a>
            <a href="/metrics/json" class="endpoint-link">📋 JSON Format</a>
            <a href="/health" class="endpoint-link">❤️ Health Check</a>
        </div>

        <button class="refresh-btn" onclick="loadMetrics()">🔄 Refresh Metrics</button>

        <div id="metrics-container">
            <p style="text-align: center; color: #666;">Loading metrics...</p>
        </div>

        <p class="timestamp" id="timestamp"></p>
    </div>

    <script>
        function formatValue(value) {
            if (typeof value === 'number') {
                if (value > 1000000) return (value / 1000000).toFixed(2) + 'M';
                if (value > 1000) return (value / 1000).toFixed(2) + 'K';
                if (value < 1 && value > 0) return value.toFixed(3);
                return value.toFixed(2);
            }
            return value;
        }

        function getValueClass(label, value) {
            if (typeof value !== 'number') return '';
            if (label.includes('error') || label.includes('denied')) {
                return value > 0 ? 'error' : 'good';
            }
            if (label.includes('latency') || label.includes('duration')) {
                if (value < 100) return 'good';
                if (value < 1000) return 'warning';
                return 'error';
            }
            return '';
        }

        function renderMetrics(data) {
            const container = document.getElementById('metrics-container');
            const summary = data.summary || {};
            const backend = data.backend || {};
            const kernel = data.kernel || {};

            let html = '<div class="grid">';

            // Summary Card
            html += '<div class="card"><h2>📈 Summary</h2>';
            html += '<div class="metric"><span class="metric-label">Total Requests</span><span class="metric-value">' + formatValue(summary.total_requests || 0) + '</span></div>';
            html += '<div class="metric"><span class="metric-label">Avg Latency</span><span class="metric-value ' + getValueClass('latency', summary.average_latency_ms) + '">' + formatValue(summary.average_latency_ms || 0) + ' ms</span></div>';
            html += '<div class="metric"><span class="metric-label">Error Rate</span><span class="metric-value ' + (summary.error_rate > 0.01 ? 'error' : 'good') + '">' + (summary.error_rate * 100).toFixed(2) + '%</span></div>';
            html += '<div class="metric"><span class="metric-label">Active Connections</span><span class="metric-value">' + formatValue(summary.active_connections || 0) + '</span></div>';
            html += '<div class="metric"><span class="metric-label">Uptime</span><span class="metric-value good">' + formatValue(summary.uptime_seconds || 0) + ' s</span></div>';
            html += '</div>';

            // Backend Card
            if (Object.keys(backend).length > 0) {
                html += '<div class="card"><h2>🔧 Backend (Go)</h2>';
                for (const [key, value] of Object.entries(backend)) {
                    const label = key.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase());
                    html += '<div class="metric"><span class="metric-label">' + label + '</span><span class="metric-value ' + getValueClass(key, value) + '">' + formatValue(value) + '</span></div>';
                }
                html += '</div>';
            }

            // Kernel Card
            if (Object.keys(kernel).length > 0) {
                html += '<div class="card"><h2>⚙️ Kernel (Rust)</h2>';
                for (const [key, value] of Object.entries(kernel)) {
                    const label = key.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase());
                    html += '<div class="metric"><span class="metric-label">' + label + '</span><span class="metric-value ' + getValueClass(key, value) + '">' + formatValue(value) + '</span></div>';
                }
                html += '</div>';
            }

            html += '</div>';
            container.innerHTML = html;

            document.getElementById('timestamp').textContent =
                'Last updated: ' + new Date(data.timestamp).toLocaleString();
        }

        function loadMetrics() {
            fetch('/metrics/json')
                .then(response => response.json())
                .then(data => renderMetrics(data))
                .catch(error => {
                    console.error('Error loading metrics:', error);
                    document.getElementById('metrics-container').innerHTML =
                        '<p style="text-align: center; color: #f87171;">Error loading metrics</p>';
                });
        }

        // Auto-refresh every 5 seconds
        loadMetrics();
        setInterval(loadMetrics, 5000);
    </script>
</body>
</html>`

	c.Header("Content-Type", "text/html; charset=utf-8")
	c.String(http.StatusOK, html)
}

// getBackendMetrics collects backend-specific metrics
func (ma *MetricsAggregator) getBackendMetrics() map[string]interface{} {
	snapshot := ma.metrics.GetSnapshot()
	uptime := ma.metrics.GetUptimeSeconds()

	return map[string]interface{}{
		"status":             "operational",
		"total_requests":     snapshot.TotalRequests,
		"total_errors":       snapshot.TotalErrors,
		"active_apps":        snapshot.ActiveApps,
		"active_connections": snapshot.ActiveConnections,
		"uptime_seconds":     uptime,
	}
}

// getKernelMetrics fetches metrics from kernel via gRPC
func (ma *MetricsAggregator) getKernelMetrics() map[string]interface{} {
	// This would call kernel gRPC endpoint to get metrics
	// For now, return placeholder
	return map[string]interface{}{
		"status": "operational",
	}
}

// getAIMetrics fetches metrics from AI service with circuit breaker protection
func (ma *MetricsAggregator) getAIMetrics() map[string]interface{} {
	result, err := ma.breaker.Execute(func() (interface{}, error) {
		// AI service HTTP metrics server runs on port 50053
		resp, err := ma.httpClient.Get("http://localhost:50053/metrics/json")
		if err != nil {
			return nil, err
		}
		defer resp.Body.Close()

		if resp.StatusCode != http.StatusOK {
			return nil, fmt.Errorf("AI service returned status %d", resp.StatusCode)
		}

		body, err := io.ReadAll(resp.Body)
		if err != nil {
			return nil, err
		}

		var metrics map[string]interface{}
		if err := json.Unmarshal(body, &metrics); err != nil {
			return nil, err
		}

		return metrics, nil
	})

	if err != nil {
		// AI service not available or circuit breaker open
		return nil
	}

	return result.(map[string]interface{})
}

// calculateSummary computes high-level summary metrics
func (ma *MetricsAggregator) calculateSummary() MetricsSummary {
	snapshot := ma.metrics.GetSnapshot()
	uptime := ma.metrics.GetUptimeSeconds()

	// Calculate average latency
	var avgLatency float64
	if snapshot.RequestCount > 0 {
		avgLatency = (snapshot.TotalDuration / float64(snapshot.RequestCount)) * 1000 // Convert to ms
	}

	// Calculate error rate
	var errorRate float64
	if snapshot.TotalRequests > 0 {
		errorRate = float64(snapshot.TotalErrors) / float64(snapshot.TotalRequests)
	}

	return MetricsSummary{
		TotalRequests:     snapshot.TotalRequests,
		AverageLatencyMs:  avgLatency,
		ErrorRate:         errorRate,
		ActiveConnections: int(snapshot.ActiveConnections),
		UptimeSeconds:     uptime,
	}
}

// ProxyKernelMetrics proxies kernel metrics endpoint
func (ma *MetricsAggregator) ProxyKernelMetrics(c *gin.Context) {
	if ma.kernel == nil {
		c.JSON(http.StatusServiceUnavailable, gin.H{
			"error": "Kernel client not available",
		})
		return
	}

	// TODO: Implement kernel metrics proxy via gRPC
	c.JSON(http.StatusOK, gin.H{
		"status": "kernel_metrics_placeholder",
	})
}

// ProxyAIMetrics proxies AI service metrics with circuit breaker protection
func (ma *MetricsAggregator) ProxyAIMetrics(c *gin.Context) {
	result, err := ma.breaker.Execute(func() (interface{}, error) {
		resp, err := ma.httpClient.Get("http://localhost:50052/metrics")
		if err != nil {
			return nil, err
		}
		defer resp.Body.Close()

		body, err := io.ReadAll(resp.Body)
		if err != nil {
			return nil, err
		}

		// Try to parse as JSON
		var jsonData interface{}
		if err := json.Unmarshal(body, &jsonData); err == nil {
			return map[string]interface{}{
				"type": "json",
				"data": jsonData,
			}, nil
		}

		// Return as text if not JSON
		return map[string]interface{}{
			"type": "text",
			"data": string(body),
		}, nil
	})

	if err == resilience.ErrCircuitOpen {
		c.JSON(http.StatusServiceUnavailable, gin.H{
			"error": "AI service unavailable: circuit breaker open",
		})
		return
	}

	if err != nil {
		c.JSON(http.StatusServiceUnavailable, gin.H{
			"error": fmt.Sprintf("AI service unavailable: %v", err),
		})
		return
	}

	responseData := result.(map[string]interface{})
	if responseData["type"] == "json" {
		c.JSON(http.StatusOK, responseData["data"])
	} else {
		c.String(http.StatusOK, responseData["data"].(string))
	}
}
