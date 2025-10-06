package http

import (
	"net/http"

	"github.com/gin-gonic/gin"
)

// ScheduleNext schedules the next process
func (h *Handlers) ScheduleNext(c *gin.Context) {
	ctx := c.Request.Context()

	nextPID, err := h.kernel.ScheduleNext(ctx)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"success": false,
			"error":   err.Error(),
		})
		return
	}

	if nextPID == nil {
		c.JSON(http.StatusOK, gin.H{
			"success":  true,
			"next_pid": nil,
			"message":  "No processes available to schedule",
		})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"success":  true,
		"next_pid": *nextPID,
	})
}

// GetSchedulerStats retrieves scheduler statistics
func (h *Handlers) GetSchedulerStats(c *gin.Context) {
	ctx := c.Request.Context()

	stats, err := h.kernel.GetSchedulerStats(ctx)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"success": false,
			"error":   err.Error(),
		})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"success": true,
		"stats": gin.H{
			"total_scheduled":  stats.TotalScheduled,
			"context_switches": stats.ContextSwitches,
			"preemptions":      stats.Preemptions,
			"active_processes": stats.ActiveProcesses,
			"policy":           stats.Policy,
			"quantum_micros":   stats.QuantumMicros,
		},
	})
}

// SetSchedulingPolicy changes the scheduling policy
func (h *Handlers) SetSchedulingPolicy(c *gin.Context) {
	ctx := c.Request.Context()

	var req struct {
		Policy string `json:"policy" binding:"required"`
	}

	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"success": false,
			"error":   "Invalid request: " + err.Error(),
		})
		return
	}

	// Validate policy
	validPolicies := map[string]bool{
		"RoundRobin": true,
		"Priority":   true,
		"Fair":       true,
	}

	if !validPolicies[req.Policy] {
		c.JSON(http.StatusBadRequest, gin.H{
			"success": false,
			"error":   "Invalid policy. Must be RoundRobin, Priority, or Fair",
		})
		return
	}

	if err := h.kernel.SetSchedulingPolicy(ctx, req.Policy); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{
			"success": false,
			"error":   err.Error(),
		})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"success": true,
		"policy":  req.Policy,
		"message": "Scheduling policy updated successfully",
	})
}
