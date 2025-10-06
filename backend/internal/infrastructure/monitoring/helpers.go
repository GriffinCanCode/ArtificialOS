package monitoring

import (
	"strings"
	"time"
)

// GetMetricsPrometheus returns metrics in Prometheus format
func (m *Metrics) GetMetricsPrometheus() string {
	var sb strings.Builder

	// Note: Prometheus client_golang already handles exposition format
	// This is a placeholder for custom metrics if needed
	sb.WriteString("# AgentOS Backend Metrics\n")
	sb.WriteString("# All metrics are collected via Prometheus client_golang\n")
	sb.WriteString("# Access via /metrics endpoint\n")

	return sb.String()
}

// GetSnapshot returns the current metrics snapshot for JSON API
func (m *Metrics) GetSnapshot() MetricsSnapshot {
	m.mu.RLock()
	defer m.mu.RUnlock()
	return m.snapshot
}

// GetUptimeSeconds returns the uptime in seconds
func (m *Metrics) GetUptimeSeconds() float64 {
	return time.Since(m.startTime).Seconds()
}

// SetWSConnections sets the number of active WebSocket connections
func (m *Metrics) SetWSConnections(count int) {
	m.WSConnections.Set(float64(count))
	m.mu.Lock()
	m.snapshot.ActiveConnections = int64(count)
	m.mu.Unlock()
}
