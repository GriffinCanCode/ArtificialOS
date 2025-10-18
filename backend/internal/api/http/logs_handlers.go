package http

import (
	"net/http"
	"time"

	"github.com/gin-gonic/gin"
	"go.uber.org/zap"
)

// UILogEntry represents a log entry from the UI
type UILogEntry struct {
	ID        string                 `json:"id"`
	Level     string                 `json:"level"`
	Message   string                 `json:"message"`
	Context   map[string]interface{} `json:"context"`
	Timestamp string                 `json:"timestamp"`
	Priority  int                    `json:"priority"`
}

// UILogStreamRequest represents a batch of logs from the UI
type UILogStreamRequest struct {
	Source    string       `json:"source"`    // "ui"
	Entries   []UILogEntry `json:"entries"`   // Log entries
	Timestamp int64        `json:"timestamp"` // Request timestamp
}

// StreamLogs handles streaming logs from the UI frontend
func (h *Handlers) StreamLogs(c *gin.Context) {
	var req UILogStreamRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid log request format"})
		return
	}

	// Validate source
	if req.Source != "ui" {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid log source"})
		return
	}

	// Validate entries
	if len(req.Entries) == 0 {
		c.JSON(http.StatusBadRequest, gin.H{"error": "No log entries provided"})
		return
	}

	// Process each log entry
	processed := 0
	for _, entry := range req.Entries {
		if err := h.processUILogEntry(entry); err != nil {
			// Log the error but continue processing other entries
			h.logProcessingError(entry, err)
		} else {
			processed++
		}
	}

	c.JSON(http.StatusOK, gin.H{
		"success":           true,
		"entries_received":  len(req.Entries),
		"entries_processed": processed,
		"timestamp":         time.Now().Unix(),
	})
}

// processUILogEntry processes a single UI log entry
func (h *Handlers) processUILogEntry(entry UILogEntry) error {
	// Create structured fields from context
	fields := make([]zap.Field, 0, len(entry.Context)+4)

	// Add standard fields
	fields = append(fields,
		zap.String("ui_log_id", entry.ID),
		zap.String("source", "ui"),
		zap.String("ui_timestamp", entry.Timestamp),
		zap.Int("priority", entry.Priority),
	)

	// Add context fields
	for key, value := range entry.Context {
		switch v := value.(type) {
		case string:
			fields = append(fields, zap.String(key, v))
		case int:
			fields = append(fields, zap.Int(key, v))
		case int64:
			fields = append(fields, zap.Int64(key, v))
		case float64:
			fields = append(fields, zap.Float64(key, v))
		case bool:
			fields = append(fields, zap.Bool(key, v))
		default:
			fields = append(fields, zap.Any(key, v))
		}
	}

	// Create logger from tracer (which has structured logging)
	logger := h.tracer.Logger()

	// Log based on level
	switch entry.Level {
	case "error":
		logger.Error(entry.Message, fields...)
	case "warn":
		logger.Warn(entry.Message, fields...)
	case "info":
		logger.Info(entry.Message, fields...)
	case "debug":
		logger.Debug(entry.Message, fields...)
	case "verbose":
		// Treat verbose as debug
		logger.Debug("[VERBOSE] "+entry.Message, fields...)
	default:
		logger.Info(entry.Message, fields...)
	}

	return nil
}

// logProcessingError logs errors that occur while processing UI log entries
func (h *Handlers) logProcessingError(entry UILogEntry, err error) {
	logger := h.tracer.Logger()
	logger.Error("Failed to process UI log entry",
		zap.Error(err),
		zap.String("ui_log_id", entry.ID),
		zap.String("ui_level", entry.Level),
		zap.String("ui_message", entry.Message),
		zap.String("source", "ui_log_processor"),
	)
}

// GetLogs retrieves recent logs (can be used for debugging)
func (h *Handlers) GetLogs(c *gin.Context) {
	// This would typically integrate with the system provider's log buffer
	// For now, return a simple response
	c.JSON(http.StatusOK, gin.H{
		"message": "Log retrieval not implemented yet",
		"hint":    "Use system.get_logs service tool for log access",
	})
}
