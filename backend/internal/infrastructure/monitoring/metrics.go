package monitoring

import (
	"sync"
	"time"

	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promauto"
)

// Metrics holds all Prometheus metrics
type Metrics struct {
	// HTTP metrics
	RequestsTotal   *prometheus.CounterVec
	RequestDuration *prometheus.HistogramVec
	RequestSize     *prometheus.HistogramVec
	ResponseSize    *prometheus.HistogramVec

	// Application metrics
	AppsActive prometheus.Gauge
	AppsTotal  prometheus.Counter

	// Service metrics
	ServiceCalls    *prometheus.CounterVec
	ServiceDuration *prometheus.HistogramVec
	ServiceErrors   *prometheus.CounterVec

	// gRPC metrics
	GRPCCalls    *prometheus.CounterVec
	GRPCDuration *prometheus.HistogramVec
	GRPCErrors   *prometheus.CounterVec

	// Session metrics
	SessionsActive   prometheus.Gauge
	SessionsSaved    prometheus.Counter
	SessionsRestored prometheus.Counter

	// Registry metrics
	RegistryApps prometheus.Gauge

	// WebSocket metrics
	WSConnections prometheus.Gauge
	WSMessages    *prometheus.CounterVec

	// System metrics
	Uptime    prometheus.Gauge
	startTime time.Time

	// Snapshot for JSON API - track current values
	snapshot MetricsSnapshot

	mu sync.RWMutex
}

// MetricsSnapshot holds current metric values for JSON API
type MetricsSnapshot struct {
	TotalRequests     int64
	TotalErrors       int64
	ActiveApps        int64
	ActiveConnections int64
	TotalDuration     float64 // sum of all request durations
	RequestCount      int64   // count for averaging
}

// NewMetrics creates a new metrics collector
func NewMetrics() *Metrics {
	m := &Metrics{
		startTime: time.Now(),

		// HTTP metrics
		RequestsTotal: promauto.NewCounterVec(
			prometheus.CounterOpts{
				Name: "backend_http_requests_total",
				Help: "Total number of HTTP requests",
			},
			[]string{"method", "path", "status"},
		),
		RequestDuration: promauto.NewHistogramVec(
			prometheus.HistogramOpts{
				Name:    "backend_http_request_duration_seconds",
				Help:    "HTTP request duration in seconds",
				Buckets: []float64{.001, .005, .01, .025, .05, .1, .25, .5, 1, 2.5, 5, 10},
			},
			[]string{"method", "path"},
		),
		RequestSize: promauto.NewHistogramVec(
			prometheus.HistogramOpts{
				Name:    "backend_http_request_size_bytes",
				Help:    "HTTP request size in bytes",
				Buckets: []float64{100, 1000, 10000, 100000, 1000000, 10000000},
			},
			[]string{"method", "path"},
		),
		ResponseSize: promauto.NewHistogramVec(
			prometheus.HistogramOpts{
				Name:    "backend_http_response_size_bytes",
				Help:    "HTTP response size in bytes",
				Buckets: []float64{100, 1000, 10000, 100000, 1000000, 10000000},
			},
			[]string{"method", "path"},
		),

		// Application metrics
		AppsActive: promauto.NewGauge(
			prometheus.GaugeOpts{
				Name: "backend_apps_active",
				Help: "Number of active applications",
			},
		),
		AppsTotal: promauto.NewCounter(
			prometheus.CounterOpts{
				Name: "backend_apps_total",
				Help: "Total number of applications created",
			},
		),

		// Service metrics
		ServiceCalls: promauto.NewCounterVec(
			prometheus.CounterOpts{
				Name: "backend_service_calls_total",
				Help: "Total number of service calls",
			},
			[]string{"service", "method", "status"},
		),
		ServiceDuration: promauto.NewHistogramVec(
			prometheus.HistogramOpts{
				Name:    "backend_service_duration_seconds",
				Help:    "Service call duration in seconds",
				Buckets: []float64{.001, .005, .01, .025, .05, .1, .25, .5, 1, 2.5, 5},
			},
			[]string{"service", "method"},
		),
		ServiceErrors: promauto.NewCounterVec(
			prometheus.CounterOpts{
				Name: "backend_service_errors_total",
				Help: "Total number of service errors",
			},
			[]string{"service", "method", "error_type"},
		),

		// gRPC metrics
		GRPCCalls: promauto.NewCounterVec(
			prometheus.CounterOpts{
				Name: "backend_grpc_calls_total",
				Help: "Total number of gRPC calls",
			},
			[]string{"service", "method", "status"},
		),
		GRPCDuration: promauto.NewHistogramVec(
			prometheus.HistogramOpts{
				Name:    "backend_grpc_duration_seconds",
				Help:    "gRPC call duration in seconds",
				Buckets: []float64{.001, .005, .01, .025, .05, .1, .25, .5, 1, 2.5, 5},
			},
			[]string{"service", "method"},
		),
		GRPCErrors: promauto.NewCounterVec(
			prometheus.CounterOpts{
				Name: "backend_grpc_errors_total",
				Help: "Total number of gRPC errors",
			},
			[]string{"service", "method", "code"},
		),

		// Session metrics
		SessionsActive: promauto.NewGauge(
			prometheus.GaugeOpts{
				Name: "backend_sessions_active",
				Help: "Number of active sessions",
			},
		),
		SessionsSaved: promauto.NewCounter(
			prometheus.CounterOpts{
				Name: "backend_sessions_saved_total",
				Help: "Total number of sessions saved",
			},
		),
		SessionsRestored: promauto.NewCounter(
			prometheus.CounterOpts{
				Name: "backend_sessions_restored_total",
				Help: "Total number of sessions restored",
			},
		),

		// Registry metrics
		RegistryApps: promauto.NewGauge(
			prometheus.GaugeOpts{
				Name: "backend_registry_apps",
				Help: "Number of apps in registry",
			},
		),

		// WebSocket metrics
		WSConnections: promauto.NewGauge(
			prometheus.GaugeOpts{
				Name: "backend_ws_connections",
				Help: "Number of active WebSocket connections",
			},
		),
		WSMessages: promauto.NewCounterVec(
			prometheus.CounterOpts{
				Name: "backend_ws_messages_total",
				Help: "Total number of WebSocket messages",
			},
			[]string{"direction", "type"},
		),

		// System metrics
		Uptime: promauto.NewGauge(
			prometheus.GaugeOpts{
				Name: "backend_uptime_seconds",
				Help: "Backend uptime in seconds",
			},
		),
	}

	// Start uptime updater
	go m.updateUptime()

	return m
}

// updateUptime continuously updates the uptime metric
func (m *Metrics) updateUptime() {
	ticker := time.NewTicker(time.Second)
	defer ticker.Stop()

	for range ticker.C {
		m.Uptime.Set(time.Since(m.startTime).Seconds())
	}
}

// RecordHTTPRequest records an HTTP request
func (m *Metrics) RecordHTTPRequest(method, path, status string, duration time.Duration, reqSize, respSize int64) {
	m.RequestsTotal.WithLabelValues(method, path, status).Inc()
	m.RequestDuration.WithLabelValues(method, path).Observe(duration.Seconds())
	m.RequestSize.WithLabelValues(method, path).Observe(float64(reqSize))
	m.ResponseSize.WithLabelValues(method, path).Observe(float64(respSize))

	// Update snapshot
	m.mu.Lock()
	m.snapshot.TotalRequests++
	m.snapshot.TotalDuration += duration.Seconds()
	m.snapshot.RequestCount++
	if status[0] == '4' || status[0] == '5' {
		m.snapshot.TotalErrors++
	}
	m.mu.Unlock()
}

// RecordServiceCall records a service call
func (m *Metrics) RecordServiceCall(service, method, status string, duration time.Duration) {
	m.ServiceCalls.WithLabelValues(service, method, status).Inc()
	m.ServiceDuration.WithLabelValues(service, method).Observe(duration.Seconds())
}

// RecordServiceError records a service error
func (m *Metrics) RecordServiceError(service, method, errorType string) {
	m.ServiceErrors.WithLabelValues(service, method, errorType).Inc()
}

// RecordGRPCCall records a gRPC call
func (m *Metrics) RecordGRPCCall(service, method, status string, duration time.Duration) {
	m.GRPCCalls.WithLabelValues(service, method, status).Inc()
	m.GRPCDuration.WithLabelValues(service, method).Observe(duration.Seconds())
}

// RecordGRPCError records a gRPC error
func (m *Metrics) RecordGRPCError(service, method, code string) {
	m.GRPCErrors.WithLabelValues(service, method, code).Inc()
}

// RecordWSMessage records a WebSocket message
func (m *Metrics) RecordWSMessage(direction, msgType string) {
	m.WSMessages.WithLabelValues(direction, msgType).Inc()
}

// SetAppsActive sets the number of active applications
func (m *Metrics) SetAppsActive(count int) {
	m.AppsActive.Set(float64(count))
	m.mu.Lock()
	m.snapshot.ActiveApps = int64(count)
	m.mu.Unlock()
}

// IncAppsTotal increments the total applications counter
func (m *Metrics) IncAppsTotal() {
	m.AppsTotal.Inc()
}

// SetSessionsActive sets the number of active sessions
func (m *Metrics) SetSessionsActive(count int) {
	m.SessionsActive.Set(float64(count))
}

// IncSessionsSaved increments the sessions saved counter
func (m *Metrics) IncSessionsSaved() {
	m.SessionsSaved.Inc()
}

// IncSessionsRestored increments the sessions restored counter
func (m *Metrics) IncSessionsRestored() {
	m.SessionsRestored.Inc()
}

// SetRegistryApps sets the number of apps in registry
func (m *Metrics) SetRegistryApps(count int) {
	m.RegistryApps.Set(float64(count))
}

// IncWSConnections increments WebSocket connections
func (m *Metrics) IncWSConnections() {
	m.WSConnections.Inc()
}

// DecWSConnections decrements WebSocket connections
func (m *Metrics) DecWSConnections() {
	m.WSConnections.Dec()
}
