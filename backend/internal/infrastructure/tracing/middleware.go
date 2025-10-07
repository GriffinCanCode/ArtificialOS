package tracing

import (
	"context"
	"time"

	"github.com/gin-gonic/gin"
	"google.golang.org/grpc"
	"google.golang.org/grpc/metadata"
)

// HTTPMiddleware creates Gin middleware for HTTP tracing
func HTTPMiddleware(tracer *Tracer) gin.HandlerFunc {
	return func(c *gin.Context) {
		// Extract trace context from headers
		headers := map[string]string{
			"X-Trace-ID": c.GetHeader("X-Trace-ID"),
			"X-Span-ID":  c.GetHeader("X-Span-ID"),
		}

		traceID, parentID := ExtractTraceContext(headers)

		ctx := c.Request.Context()
		if traceID != "" {
			ctx = context.WithValue(ctx, traceIDKey, traceID)
		}
		if parentID != "" {
			ctx = context.WithValue(ctx, spanIDKey, parentID)
		}

		// Start span
		span, ctx := tracer.StartSpan(ctx, c.FullPath())
		span.SetTag("http.method", c.Request.Method)
		span.SetTag("http.url", c.Request.URL.String())
		span.SetTag("http.host", c.Request.Host)

		// Update request context
		c.Request = c.Request.WithContext(ctx)

		// Inject trace context into response headers
		c.Header("X-Trace-ID", string(span.TraceID))
		c.Header("X-Span-ID", string(span.SpanID))

		// Process request
		start := time.Now()
		c.Next()
		span.Duration = time.Since(start)

		// Record response
		span.SetStatus(c.Writer.Status())
		span.SetTag("http.status", string(rune(c.Writer.Status())))

		if len(c.Errors) > 0 {
			span.SetError(c.Errors.Last())
		}

		span.Finish()
		tracer.Submit(span)
	}
}

// GRPCUnaryInterceptor creates a gRPC unary interceptor for tracing
func GRPCUnaryInterceptor(tracer *Tracer) grpc.UnaryServerInterceptor {
	return func(
		ctx context.Context,
		req interface{},
		info *grpc.UnaryServerInfo,
		handler grpc.UnaryHandler,
	) (interface{}, error) {
		// Extract trace context from metadata
		md, ok := metadata.FromIncomingContext(ctx)
		if ok {
			headers := make(map[string]string)
			if vals := md.Get("x-trace-id"); len(vals) > 0 {
				headers["X-Trace-ID"] = vals[0]
			}
			if vals := md.Get("x-span-id"); len(vals) > 0 {
				headers["X-Span-ID"] = vals[0]
			}

			traceID, parentID := ExtractTraceContext(headers)
			if traceID != "" {
				ctx = context.WithValue(ctx, traceIDKey, traceID)
			}
			if parentID != "" {
				ctx = context.WithValue(ctx, spanIDKey, parentID)
			}
		}

		// Start span
		span, ctx := tracer.StartSpan(ctx, info.FullMethod)
		span.SetTag("rpc.system", "grpc")
		span.SetTag("rpc.method", info.FullMethod)

		// Process request
		resp, err := handler(ctx, req)

		// Record result
		if err != nil {
			span.SetError(err)
		} else {
			span.SetStatus(200)
		}

		span.Finish()
		tracer.Submit(span)

		return resp, err
	}
}

// GRPCStreamInterceptor creates a gRPC stream interceptor for tracing
func GRPCStreamInterceptor(tracer *Tracer) grpc.StreamServerInterceptor {
	return func(
		srv interface{},
		ss grpc.ServerStream,
		info *grpc.StreamServerInfo,
		handler grpc.StreamHandler,
	) error {
		ctx := ss.Context()

		// Extract trace context
		md, ok := metadata.FromIncomingContext(ctx)
		if ok {
			headers := make(map[string]string)
			if vals := md.Get("x-trace-id"); len(vals) > 0 {
				headers["X-Trace-ID"] = vals[0]
			}
			if vals := md.Get("x-span-id"); len(vals) > 0 {
				headers["X-Span-ID"] = vals[0]
			}

			traceID, parentID := ExtractTraceContext(headers)
			if traceID != "" {
				ctx = context.WithValue(ctx, traceIDKey, traceID)
			}
			if parentID != "" {
				ctx = context.WithValue(ctx, spanIDKey, parentID)
			}
		}

		// Start span
		span, ctx := tracer.StartSpan(ctx, info.FullMethod)
		span.SetTag("rpc.system", "grpc")
		span.SetTag("rpc.method", info.FullMethod)
		span.SetTag("rpc.streaming", "true")

		// Wrap stream with traced context
		wrapped := &tracedServerStream{
			ServerStream: ss,
			ctx:          ctx,
		}

		// Process stream
		err := handler(srv, wrapped)

		// Record result
		if err != nil {
			span.SetError(err)
		} else {
			span.SetStatus(200)
		}

		span.Finish()
		tracer.Submit(span)

		return err
	}
}

// tracedServerStream wraps grpc.ServerStream with tracing context
type tracedServerStream struct {
	grpc.ServerStream
	ctx context.Context
}

func (s *tracedServerStream) Context() context.Context {
	return s.ctx
}

// GRPCClientInterceptor creates a gRPC client interceptor for trace propagation
func GRPCClientInterceptor(tracer *Tracer) grpc.UnaryClientInterceptor {
	return func(
		ctx context.Context,
		method string,
		req, reply interface{},
		cc *grpc.ClientConn,
		invoker grpc.UnaryInvoker,
		opts ...grpc.CallOption,
	) error {
		// Start client span
		span, ctx := tracer.StartSpan(ctx, method)
		span.SetTag("rpc.system", "grpc")
		span.SetTag("rpc.method", method)
		span.SetTag("span.kind", "client")

		// Inject trace context into metadata
		headers := make(map[string]string)
		InjectTraceContext(ctx, headers)

		md := metadata.New(headers)
		ctx = metadata.NewOutgoingContext(ctx, md)

		// Call remote service
		err := invoker(ctx, method, req, reply, cc, opts...)

		// Record result
		if err != nil {
			span.SetError(err)
		} else {
			span.SetStatus(200)
		}

		span.Finish()
		tracer.Submit(span)

		return err
	}
}
