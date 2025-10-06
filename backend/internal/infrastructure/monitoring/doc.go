/*
Package monitoring provides performance monitoring and metrics collection.

# Overview

This package implements Prometheus-based metrics collection for the backend
service, tracking HTTP requests, service calls, gRPC operations, and system
metrics.

# Features

- HTTP request metrics (latency, throughput, size)
- Service call metrics (duration, errors)
- gRPC call metrics (latency, status codes)
- Application lifecycle metrics
- Session management metrics
- WebSocket connection metrics
- System metrics (uptime, resource usage)

# Usage

	// Create metrics collector
	metrics := monitoring.NewMetrics()

	// Add middleware to Gin router
	router.Use(monitoring.Middleware(metrics))

	// Record custom metrics
	metrics.SetAppsActive(5)
	metrics.IncAppsTotal()

	// Time operations
	timer := monitoring.NewTimer(metrics, "storage", "write")
	// ... perform operation ...
	timer.Stop("success")

# Metrics Endpoint

Expose metrics via the standard Prometheus endpoint:

	import "github.com/prometheus/client_golang/prometheus/promhttp"
	router.GET("/metrics", gin.WrapH(promhttp.Handler()))
*/
package monitoring
