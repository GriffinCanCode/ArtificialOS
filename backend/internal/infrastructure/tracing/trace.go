package tracing

import (
	"context"
	"fmt"
	"time"

	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/id"
	"go.uber.org/zap"
)

// TraceID represents a unique trace identifier
type TraceID string

// SpanID represents a unique span identifier
type SpanID string

// Span represents a single operation in a trace
type Span struct {
	TraceID    TraceID
	SpanID     SpanID
	ParentID   SpanID
	Name       string
	Service    string
	StartTime  time.Time
	EndTime    time.Time
	Duration   time.Duration
	Tags       map[string]string
	Logs       []LogEntry
	Error      error
	StatusCode int
}

// LogEntry represents a log within a span
type LogEntry struct {
	Timestamp time.Time
	Message   string
	Fields    map[string]interface{}
}

// Tracer manages distributed tracing
type Tracer struct {
	service string
	logger  *zap.Logger
	spans   chan *Span
}

// New creates a new tracer instance
func New(service string, logger *zap.Logger) *Tracer {
	t := &Tracer{
		service: service,
		logger:  logger,
		spans:   make(chan *Span, 1000),
	}

	// Start span collector
	go t.collectSpans()

	return t
}

// StartSpan creates a new span
func (t *Tracer) StartSpan(ctx context.Context, name string) (*Span, context.Context) {
	traceID, _ := ctx.Value(traceIDKey).(TraceID)
	if traceID == "" {
		traceID = TraceID(id.NewRequestID())
	}

	parentID, _ := ctx.Value(spanIDKey).(SpanID)

	span := &Span{
		TraceID:   traceID,
		SpanID:    SpanID(id.NewRequestID()),
		ParentID:  parentID,
		Name:      name,
		Service:   t.service,
		StartTime: time.Now(),
		Tags:      make(map[string]string),
		Logs:      []LogEntry{},
	}

	newCtx := context.WithValue(ctx, traceIDKey, traceID)
	newCtx = context.WithValue(newCtx, spanIDKey, span.SpanID)

	return span, newCtx
}

// Finish marks the span as complete
func (s *Span) Finish() {
	s.EndTime = time.Now()
	s.Duration = s.EndTime.Sub(s.StartTime)
}

// SetTag adds a tag to the span
func (s *Span) SetTag(key, value string) {
	s.Tags[key] = value
}

// SetError records an error in the span
func (s *Span) SetError(err error) {
	s.Error = err
	s.StatusCode = 500
}

// SetStatus sets the HTTP status code
func (s *Span) SetStatus(code int) {
	s.StatusCode = code
}

// Log adds a log entry to the span
func (s *Span) Log(message string, fields map[string]interface{}) {
	s.Logs = append(s.Logs, LogEntry{
		Timestamp: time.Now(),
		Message:   message,
		Fields:    fields,
	})
}

// collectSpans processes completed spans
func (t *Tracer) collectSpans() {
	for span := range t.spans {
		t.processSpan(span)
	}
}

// processSpan logs and exports span data
func (t *Tracer) processSpan(span *Span) {
	fields := []zap.Field{
		zap.String("trace_id", string(span.TraceID)),
		zap.String("span_id", string(span.SpanID)),
		zap.String("operation", span.Name),
		zap.Duration("duration", span.Duration),
		zap.String("service", span.Service),
	}

	if span.ParentID != "" {
		fields = append(fields, zap.String("parent_id", string(span.ParentID)))
	}

	if span.Error != nil {
		fields = append(fields, zap.Error(span.Error))
		t.logger.Error("span completed with error", fields...)
	} else {
		t.logger.Info("span completed", fields...)
	}
}

// Submit sends a span to the collector
func (t *Tracer) Submit(span *Span) {
	select {
	case t.spans <- span:
	default:
		t.logger.Warn("span buffer full, dropping span",
			zap.String("trace_id", string(span.TraceID)),
			zap.String("span_id", string(span.SpanID)),
		)
	}
}

// ExtractTraceContext extracts trace context from headers
func ExtractTraceContext(headers map[string]string) (TraceID, SpanID) {
	traceID := TraceID(headers["X-Trace-ID"])
	spanID := SpanID(headers["X-Span-ID"])
	return traceID, spanID
}

// InjectTraceContext injects trace context into headers
func InjectTraceContext(ctx context.Context, headers map[string]string) {
	if traceID, ok := ctx.Value(traceIDKey).(TraceID); ok {
		headers["X-Trace-ID"] = string(traceID)
	}
	if spanID, ok := ctx.Value(spanIDKey).(SpanID); ok {
		headers["X-Span-ID"] = string(spanID)
	}
}

// Context keys for trace propagation
type contextKey string

const (
	traceIDKey contextKey = "trace_id"
	spanIDKey  contextKey = "span_id"
)

// GetTraceID retrieves the trace ID from context
func GetTraceID(ctx context.Context) TraceID {
	if traceID, ok := ctx.Value(traceIDKey).(TraceID); ok {
		return traceID
	}
	return ""
}

// GetSpanID retrieves the span ID from context
func GetSpanID(ctx context.Context) SpanID {
	if spanID, ok := ctx.Value(spanIDKey).(SpanID); ok {
		return spanID
	}
	return ""
}

// FormatTrace returns a formatted trace string for logging
func FormatTrace(traceID TraceID, spanID SpanID) string {
	return fmt.Sprintf("[trace:%s span:%s]", traceID, spanID)
}
