package service

import (
	"fmt"
	"sort"
	"strings"
	"sync"

	"github.com/GriffinCanCode/AgentOS/backend/internal/types"
)

// Registry manages service discovery and execution
type Registry struct {
	services sync.Map
	mu       sync.RWMutex
}

// Provider interface for service implementations
type Provider interface {
	Definition() types.Service
	Execute(toolID string, params map[string]interface{}, ctx *types.Context) (*types.Result, error)
}

// NewRegistry creates a new service registry
func NewRegistry() *Registry {
	return &Registry{}
}

// Register adds a service provider
func (r *Registry) Register(provider Provider) error {
	def := provider.Definition()
	if def.ID == "" {
		return fmt.Errorf("service ID cannot be empty")
	}

	r.services.Store(def.ID, provider)
	return nil
}

// Unregister removes a service provider
func (r *Registry) Unregister(serviceID string) {
	r.services.Delete(serviceID)
}

// Get retrieves a service by ID
func (r *Registry) Get(serviceID string) (Provider, bool) {
	val, ok := r.services.Load(serviceID)
	if !ok {
		return nil, false
	}
	return val.(Provider), true
}

// List returns all registered services
func (r *Registry) List(category *types.Category) []types.Service {
	var services []types.Service
	r.services.Range(func(_, value interface{}) bool {
		provider := value.(Provider)
		def := provider.Definition()
		if category == nil || def.Category == *category {
			services = append(services, def)
		}
		return true
	})
	return services
}

// Discover finds relevant services for a given intent
func (r *Registry) Discover(intent string, limit int) []types.Service {
	type scoredService struct {
		service types.Service
		score   float64
	}

	intentLower := strings.ToLower(intent)
	var results []scoredService

	r.services.Range(func(_, value interface{}) bool {
		provider := value.(Provider)
		def := provider.Definition()
		score := r.calculateRelevance(intentLower, def)
		if score > 0 {
			results = append(results, scoredService{
				service: def,
				score:   score,
			})
		}
		return true
	})

	// Sort by score descending
	sort.Slice(results, func(i, j int) bool {
		return results[i].score > results[j].score
	})

	// Return top N
	output := make([]types.Service, 0, limit)
	for i := 0; i < len(results) && i < limit; i++ {
		output = append(output, results[i].service)
	}

	return output
}

// Execute runs a service tool
func (r *Registry) Execute(toolID string, params map[string]interface{}, ctx *types.Context) (*types.Result, error) {
	parts := strings.SplitN(toolID, ".", 2)
	if len(parts) < 2 {
		return &types.Result{
			Success: false,
			Error:   stringPtr("invalid tool ID format"),
		}, fmt.Errorf("invalid tool ID format: %s", toolID)
	}

	serviceID := parts[0]
	provider, ok := r.Get(serviceID)
	if !ok {
		return &types.Result{
			Success: false,
			Error:   stringPtr(fmt.Sprintf("service not found: %s", serviceID)),
		}, fmt.Errorf("service not found: %s", serviceID)
	}

	return provider.Execute(toolID, params, ctx)
}

// Stats returns registry statistics
func (r *Registry) Stats() map[string]interface{} {
	var total, totalTools int
	categories := make(map[string]int)

	r.services.Range(func(_, value interface{}) bool {
		provider := value.(Provider)
		def := provider.Definition()
		total++
		totalTools += len(def.Tools)
		categories[string(def.Category)]++
		return true
	})

	return map[string]interface{}{
		"total_services": total,
		"total_tools":    totalTools,
		"categories":     categories,
	}
}

func (r *Registry) calculateRelevance(intent string, service types.Service) float64 {
	score := 0.0

	// Check service name and ID
	if strings.Contains(intent, service.ID) || strings.Contains(intent, strings.ToLower(service.Name)) {
		score += 10.0
	}

	// Check description words
	descWords := strings.Fields(strings.ToLower(service.Description))
	for _, word := range descWords {
		if strings.Contains(intent, word) {
			score += 5.0
		}
	}

	// Check capabilities
	for _, cap := range service.Capabilities {
		capClean := strings.ReplaceAll(strings.ToLower(cap), "_", " ")
		if strings.Contains(intent, capClean) {
			score += 3.0
		}
	}

	// Check category
	if strings.Contains(intent, string(service.Category)) {
		score += 2.0
	}

	return score
}

func stringPtr(s string) *string {
	return &s
}
