package browser

import (
	"context"
	"fmt"
	"sync"

	"github.com/GriffinCanCode/AgentOS/backend/internal/grpc/kernel"
	"github.com/GriffinCanCode/AgentOS/backend/internal/providers/browser/sandbox"
	"github.com/GriffinCanCode/AgentOS/backend/internal/providers/http/client"
	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
)

// Provider implements browser proxy service
type Provider struct {
	httpClient  *client.Client
	kernel      *kernel.KernelClient
	pid         uint32
	sessions    *SessionManager
	sandboxPool *sandbox.Pool
}

// SessionManager manages browser sessions per app instance
type SessionManager struct {
	mu       sync.RWMutex
	sessions map[string]*BrowserSession // key: appId
}

// BrowserSession holds state for a browser instance
type BrowserSession struct {
	appID           string
	cookies         map[string][]*Cookie
	history         []string
	userAgent       string
	referer         string
	lastScript      string
	injectedScripts []string
	mu              sync.RWMutex
}

// Cookie represents an HTTP cookie
type Cookie struct {
	Name     string
	Value    string
	Domain   string
	Path     string
	Expires  string
	Secure   bool
	HTTPOnly bool
}

// New creates a new browser proxy provider
func New(httpClient *client.Client, kernelClient *kernel.KernelClient, pid uint32) *Provider {
	// Create sandbox pool with default config
	sandboxConfig := sandbox.DefaultConfig()
	pool, err := sandbox.NewPool(sandboxConfig, 4)
	if err != nil {
		panic(fmt.Sprintf("failed to create sandbox pool: %v", err))
	}

	return &Provider{
		httpClient: httpClient,
		kernel:     kernelClient,
		pid:        pid,
		sessions: &SessionManager{
			sessions: make(map[string]*BrowserSession),
		},
		sandboxPool: pool,
	}
}

// Definition returns service definition
func (p *Provider) Definition() types.Service {
	return types.Service{
		ID:          "browser",
		Name:        "Browser Proxy",
		Category:    types.CategorySystem,
		Description: "Server-side browser proxy for rendering web content",
		Tools:       p.getTools(),
	}
}

func (p *Provider) getTools() []types.Tool {
	return []types.Tool{
		{
			ID:          "browser.navigate",
			Name:        "Navigate to URL",
			Description: "Load a web page with full rendering support (bypasses CORS)",
			Parameters: []types.Parameter{
				{Name: "url", Type: "string", Description: "URL to navigate to", Required: true},
				{Name: "session_id", Type: "string", Description: "Browser session ID", Required: false},
				{Name: "enable_js", Type: "boolean", Description: "Enable JavaScript execution", Required: false},
			},
			Returns: "object",
		},
		{
			ID:          "browser.proxy_asset",
			Name:        "Proxy Asset",
			Description: "Fetch and proxy an asset (image, CSS, JS) through the backend",
			Parameters: []types.Parameter{
				{Name: "url", Type: "string", Description: "Asset URL", Required: true},
				{Name: "session_id", Type: "string", Description: "Browser session ID", Required: false},
			},
			Returns: "object",
		},
		{
			ID:          "browser.submit_form",
			Name:        "Submit Form",
			Description: "Submit a form through the proxy",
			Parameters: []types.Parameter{
				{Name: "url", Type: "string", Description: "Form action URL", Required: true},
				{Name: "method", Type: "string", Description: "HTTP method (GET/POST)", Required: false},
				{Name: "data", Type: "object", Description: "Form data", Required: true},
				{Name: "session_id", Type: "string", Description: "Browser session ID", Required: false},
			},
			Returns: "object",
		},
		{
			ID:          "browser.get_session",
			Name:        "Get Session Info",
			Description: "Get current browser session information",
			Parameters: []types.Parameter{
				{Name: "session_id", Type: "string", Description: "Browser session ID", Required: true},
			},
			Returns: "object",
		},
		{
			ID:          "browser.execute_script",
			Name:        "Execute JavaScript",
			Description: "Execute JavaScript code in a sandboxed environment",
			Parameters: []types.Parameter{
				{Name: "script", Type: "string", Description: "JavaScript code to execute", Required: true},
				{Name: "session_id", Type: "string", Description: "Browser session ID", Required: false},
				{Name: "dom_html", Type: "string", Description: "HTML for DOM context", Required: false},
			},
			Returns: "object",
		},
		{
			ID:          "browser.inject_script",
			Name:        "Inject Script",
			Description: "Inject a script into the page context",
			Parameters: []types.Parameter{
				{Name: "script", Type: "string", Description: "Script to inject", Required: true},
				{Name: "url", Type: "string", Description: "Page URL", Required: false},
				{Name: "session_id", Type: "string", Description: "Browser session ID", Required: false},
			},
			Returns: "object",
		},
		{
			ID:          "browser.eval_expression",
			Name:        "Evaluate Expression",
			Description: "Evaluate a JavaScript expression and return result",
			Parameters: []types.Parameter{
				{Name: "expression", Type: "string", Description: "JavaScript expression", Required: true},
				{Name: "session_id", Type: "string", Description: "Browser session ID", Required: false},
			},
			Returns: "object",
		},
	}
}

// Execute routes tool calls
func (p *Provider) Execute(ctx context.Context, toolID string, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	switch toolID {
	case "browser.navigate":
		return p.Navigate(ctx, params, appCtx)
	case "browser.proxy_asset":
		return p.ProxyAsset(ctx, params, appCtx)
	case "browser.submit_form":
		return p.SubmitForm(ctx, params, appCtx)
	case "browser.get_session":
		return p.GetSessionInfo(ctx, params, appCtx)
	case "browser.execute_script":
		return p.ExecuteScript(ctx, params, appCtx)
	case "browser.inject_script":
		return p.InjectScript(ctx, params, appCtx)
	case "browser.eval_expression":
		return p.EvalExpression(ctx, params, appCtx)
	default:
		return client.Failure(fmt.Sprintf("unknown tool: %s", toolID))
	}
}

// getOrCreateSession retrieves or creates a browser session
func (p *Provider) getOrCreateSession(sessionID string, appID string) *BrowserSession {
	if sessionID == "" {
		sessionID = appID
	}

	p.sessions.mu.Lock()
	defer p.sessions.mu.Unlock()

	session, exists := p.sessions.sessions[sessionID]
	if !exists {
		session = &BrowserSession{
			appID:     appID,
			cookies:   make(map[string][]*Cookie),
			history:   []string{},
			userAgent: "Mozilla/5.0 (AgentOS Browser/2.0) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
		}
		p.sessions.sessions[sessionID] = session
	}

	return session
}
