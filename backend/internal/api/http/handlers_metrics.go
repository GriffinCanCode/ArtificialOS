package http

import (
	"time"

	"github.com/GriffinCanCode/AgentOS/backend/internal/infrastructure/monitoring"
)

// HandlerMetrics wraps handlers with metrics tracking
type HandlerMetrics struct {
	metrics *monitoring.Metrics
}

// NewHandlerMetrics creates a metrics wrapper
func NewHandlerMetrics(metrics *monitoring.Metrics) *HandlerMetrics {
	return &HandlerMetrics{metrics: metrics}
}

// TrackAppOperation tracks app-related operations
func (hm *HandlerMetrics) TrackAppOperation(operation string) func() {
	start := time.Now()
	return func() {
		duration := time.Since(start)
		hm.metrics.RecordServiceCall("app_manager", operation, "success", duration)
	}
}

// TrackServiceOperation tracks service-related operations
func (hm *HandlerMetrics) TrackServiceOperation(operation string) func() {
	start := time.Now()
	return func() {
		duration := time.Since(start)
		hm.metrics.RecordServiceCall("service_registry", operation, "success", duration)
	}
}

// TrackRegistryOperation tracks registry operations
func (hm *HandlerMetrics) TrackRegistryOperation(operation string) func() {
	start := time.Now()
	return func() {
		duration := time.Since(start)
		hm.metrics.RecordServiceCall("app_registry", operation, "success", duration)
	}
}

// TrackSessionOperation tracks session operations
func (hm *HandlerMetrics) TrackSessionOperation(operation string) func() {
	start := time.Now()
	return func() {
		duration := time.Since(start)
		hm.metrics.RecordServiceCall("session_manager", operation, "success", duration)
	}
}
