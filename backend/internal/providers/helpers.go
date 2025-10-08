package providers

import (
	"github.com/GriffinCanCode/AgentOS/backend/internal/providers/http"
	httpProvider "github.com/GriffinCanCode/AgentOS/backend/internal/providers/http"
)

// NewHTTP creates an HTTP provider without kernel integration (for testing)
// Production code should use http.NewProvider(kernelClient, pid) instead
func NewHTTP() *http.Provider {
	return httpProvider.NewProvider(nil, 0)
}
