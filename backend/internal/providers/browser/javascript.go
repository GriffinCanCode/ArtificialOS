package browser

import (
	"context"
	"fmt"

	"github.com/GriffinCanCode/AgentOS/backend/internal/providers/browser/sandbox"
	"github.com/GriffinCanCode/AgentOS/backend/internal/providers/http/client"
	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
)

// ExecuteScript executes JavaScript code in a sandboxed environment
func (p *Provider) ExecuteScript(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	// Extract parameters
	script, ok := params["script"].(string)
	if !ok || script == "" {
		return client.Failure("script parameter required")
	}

	sessionID, _ := params["session_id"].(string)
	domHTML, _ := params["dom_html"].(string)

	// Get or create session
	appID := ""
	if appCtx.AppID != nil {
		appID = *appCtx.AppID
	}
	session := p.getOrCreateSession(sessionID, appID)

	// Create DOM if HTML provided
	var dom *sandbox.DOM
	if domHTML != "" {
		dom = sandbox.NewDOM()
		// TODO: Parse HTML into DOM structure
	}

	// Execute in sandbox pool
	result, err := p.sandboxPool.Execute(ctx, script, dom)
	if err != nil {
		return client.Failure(fmt.Sprintf("execution failed: %v", err))
	}

	// Update session with execution result
	session.mu.Lock()
	session.lastScript = script
	session.mu.Unlock()

	return client.Success(map[string]interface{}{
		"value":       result.Value,
		"console":     result.Console,
		"dom_changes": result.DOMChanges,
		"duration_ms": result.Duration.Milliseconds(),
		"error":       result.Error,
	})
}

// InjectScript injects and executes a script in the page context
func (p *Provider) InjectScript(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	script, ok := params["script"].(string)
	if !ok || script == "" {
		return client.Failure("script parameter required")
	}

	url, _ := params["url"].(string)
	sessionID, _ := params["session_id"].(string)

	appID := ""
	if appCtx.AppID != nil {
		appID = *appCtx.AppID
	}
	session := p.getOrCreateSession(sessionID, appID)

	// Add script to session's injected scripts
	session.mu.Lock()
	if session.injectedScripts == nil {
		session.injectedScripts = []string{}
	}
	session.injectedScripts = append(session.injectedScripts, script)
	session.mu.Unlock()

	return client.Success(map[string]interface{}{
		"injected": true,
		"url":      url,
	})
}

// EvalExpression evaluates a JavaScript expression
func (p *Provider) EvalExpression(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	expression, ok := params["expression"].(string)
	if !ok || expression == "" {
		return client.Failure("expression parameter required")
	}

	sessionID, _ := params["session_id"].(string)
	appID := ""
	if appCtx.AppID != nil {
		appID = *appCtx.AppID
	}
	_ = p.getOrCreateSession(sessionID, appID)

	// Wrap in return statement
	script := fmt.Sprintf("(function() { return %s; })()", expression)

	// Execute in sandbox
	result, err := p.sandboxPool.Execute(ctx, script, nil)
	if err != nil {
		return client.Failure(fmt.Sprintf("evaluation failed: %v", err))
	}

	return client.Success(map[string]interface{}{
		"value": result.Value,
		"type":  fmt.Sprintf("%T", result.Value),
	})
}
