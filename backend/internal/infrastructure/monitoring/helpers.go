package monitoring

import "strings"

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
