/*
Package tracing provides distributed tracing for debugging production issues.

# Overview

This package implements lightweight distributed tracing to track requests
across multiple services (backend, AI service, kernel). It follows OpenTelemetry
concepts but with a minimal implementation tailored to the system's needs.

# Features

- Trace context propagation via HTTP headers and gRPC metadata
- Span creation and management with parent-child relationships
- Automatic trace ID generation
- HTTP and gRPC middleware for automatic instrumentation
- Structured logging integration
- Low overhead with buffered span collection

# Usage

	// Create tracer
	tracer := tracing.New("backend", logger)

	// HTTP middleware
	router.Use(tracing.HTTPMiddleware(tracer))

	// gRPC server interceptor
	server := grpc.NewServer(
		grpc.UnaryInterceptor(tracing.GRPCUnaryInterceptor(tracer)),
		grpc.StreamInterceptor(tracing.GRPCStreamInterceptor(tracer)),
	)

	// gRPC client interceptor
	conn, err := grpc.Dial(addr,
		grpc.WithUnaryInterceptor(tracing.GRPCClientInterceptor(tracer)),
	)

	// Manual span creation
	span, ctx := tracer.StartSpan(ctx, "operation")
	defer func() {
		span.Finish()
		tracer.Submit(span)
	}()

	span.SetTag("key", "value")
	span.Log("message", map[string]interface{}{"detail": "info"})

# Trace Format

Traces use standard HTTP headers for propagation:
- X-Trace-ID: Unique identifier for entire request flow
- X-Span-ID: Identifier for current operation

# Performance

The tracing system is designed for minimal overhead:
- Buffered span collection (1000 spans)
- Async span processing
- Structured logging integration
- No external dependencies
*/
package tracing
