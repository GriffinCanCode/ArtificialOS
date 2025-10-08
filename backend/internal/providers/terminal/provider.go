package terminal

import (
	"context"
	"encoding/base64"
	"fmt"

	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
)

// Provider implements terminal emulator operations
type Provider struct {
	manager *Manager
}

// NewProvider creates a new terminal provider
func NewProvider() *Provider {
	return &Provider{
		manager: NewManager(),
	}
}

// Definition returns service metadata
func (p *Provider) Definition() types.Service {
	return types.Service{
		ID:          "terminal",
		Name:        "Terminal Service",
		Description: "Full-featured terminal emulator with PTY support for interactive shell sessions",
		Category:    types.CategorySystem,
		Capabilities: []string{
			"pty",
			"shell",
			"interactive",
			"ansi",
			"sessions",
			"resize",
		},
		Tools: p.getTools(),
	}
}

// Execute routes to appropriate operation
func (p *Provider) Execute(ctx context.Context, toolID string, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	switch toolID {
	case "terminal.create_session":
		return p.createSession(params)
	case "terminal.write":
		return p.write(params)
	case "terminal.read":
		return p.read(params)
	case "terminal.resize":
		return p.resize(params)
	case "terminal.list_sessions":
		return p.listSessions()
	case "terminal.kill":
		return p.kill(params)
	case "terminal.get_session":
		return p.getSession(params)
	default:
		return nil, fmt.Errorf("unknown tool: %s", toolID)
	}
}

func (p *Provider) getTools() []types.Tool {
	return []types.Tool{
		{
			ID:          "terminal.create_session",
			Name:        "Create Terminal Session",
			Description: "Create a new interactive terminal session with PTY",
			Parameters: []types.Parameter{
				{
					Name:        "shell",
					Type:        "string",
					Description: "Shell to use (e.g., /bin/bash, /bin/zsh). Defaults to user's shell",
					Required:    false,
				},
				{
					Name:        "working_dir",
					Type:        "string",
					Description: "Initial working directory. Defaults to user's home",
					Required:    false,
				},
				{
					Name:        "cols",
					Type:        "number",
					Description: "Terminal width in columns. Defaults to 80",
					Required:    false,
				},
				{
					Name:        "rows",
					Type:        "number",
					Description: "Terminal height in rows. Defaults to 24",
					Required:    false,
				},
				{
					Name:        "env",
					Type:        "object",
					Description: "Environment variables to set",
					Required:    false,
				},
			},
			Returns: "session_info",
		},
		{
			ID:          "terminal.write",
			Name:        "Write to Terminal",
			Description: "Send input to a terminal session",
			Parameters: []types.Parameter{
				{
					Name:        "session_id",
					Type:        "string",
					Description: "Terminal session ID",
					Required:    true,
				},
				{
					Name:        "input",
					Type:        "string",
					Description: "Input to send to terminal",
					Required:    true,
				},
			},
			Returns: "success",
		},
		{
			ID:          "terminal.read",
			Name:        "Read from Terminal",
			Description: "Read buffered output from a terminal session",
			Parameters: []types.Parameter{
				{
					Name:        "session_id",
					Type:        "string",
					Description: "Terminal session ID",
					Required:    true,
				},
			},
			Returns: "output_data",
		},
		{
			ID:          "terminal.resize",
			Name:        "Resize Terminal",
			Description: "Change terminal dimensions",
			Parameters: []types.Parameter{
				{
					Name:        "session_id",
					Type:        "string",
					Description: "Terminal session ID",
					Required:    true,
				},
				{
					Name:        "cols",
					Type:        "number",
					Description: "New width in columns",
					Required:    true,
				},
				{
					Name:        "rows",
					Type:        "number",
					Description: "New height in rows",
					Required:    true,
				},
			},
			Returns: "success",
		},
		{
			ID:          "terminal.list_sessions",
			Name:        "List Terminal Sessions",
			Description: "List all active terminal sessions",
			Parameters:  []types.Parameter{},
			Returns:     "sessions_list",
		},
		{
			ID:          "terminal.get_session",
			Name:        "Get Session Info",
			Description: "Get information about a terminal session",
			Parameters: []types.Parameter{
				{
					Name:        "session_id",
					Type:        "string",
					Description: "Terminal session ID",
					Required:    true,
				},
			},
			Returns: "session_info",
		},
		{
			ID:          "terminal.kill",
			Name:        "Kill Terminal Session",
			Description: "Terminate a terminal session",
			Parameters: []types.Parameter{
				{
					Name:        "session_id",
					Type:        "string",
					Description: "Terminal session ID",
					Required:    true,
				},
			},
			Returns: "success",
		},
	}
}

func (p *Provider) createSession(params map[string]interface{}) (*types.Result, error) {
	shell, _ := params["shell"].(string)
	workingDir, _ := params["working_dir"].(string)

	cols := 80
	if c, ok := params["cols"].(float64); ok {
		cols = int(c)
	}

	rows := 24
	if r, ok := params["rows"].(float64); ok {
		rows = int(r)
	}

	env := make(map[string]string)
	if envMap, ok := params["env"].(map[string]interface{}); ok {
		for k, v := range envMap {
			if str, ok := v.(string); ok {
				env[k] = str
			}
		}
	}

	session, err := p.manager.CreateSession(shell, workingDir, cols, rows, env)
	if err != nil {
		return nil, err
	}

	sessionData := map[string]interface{}{
		"id":          session.ID,
		"shell":       session.Shell,
		"working_dir": session.WorkingDir,
		"cols":        session.Cols,
		"rows":        session.Rows,
		"started_at":  session.StartedAt,
		"active":      session.Active,
	}

	return &types.Result{
		Success: true,
		Data:    sessionData,
	}, nil
}

func (p *Provider) write(params map[string]interface{}) (*types.Result, error) {
	sessionID, ok := params["session_id"].(string)
	if !ok {
		return nil, fmt.Errorf("session_id is required")
	}

	input, ok := params["input"].(string)
	if !ok {
		return nil, fmt.Errorf("input is required")
	}

	err := p.manager.Write(sessionID, []byte(input))
	if err != nil {
		return nil, err
	}

	return &types.Result{
		Success: true,
		Data:    map[string]interface{}{"success": true},
	}, nil
}

func (p *Provider) read(params map[string]interface{}) (*types.Result, error) {
	sessionID, ok := params["session_id"].(string)
	if !ok {
		return nil, fmt.Errorf("session_id is required")
	}

	output, err := p.manager.Read(sessionID)
	if err != nil {
		return nil, err
	}

	// Encode output as base64 to handle binary data
	encoded := base64.StdEncoding.EncodeToString(output)

	return &types.Result{
		Success: true,
		Data: map[string]interface{}{
			"output":        string(output),
			"output_base64": encoded,
			"length":        len(output),
		},
	}, nil
}

func (p *Provider) resize(params map[string]interface{}) (*types.Result, error) {
	sessionID, ok := params["session_id"].(string)
	if !ok {
		return nil, fmt.Errorf("session_id is required")
	}

	cols, ok := params["cols"].(float64)
	if !ok {
		return nil, fmt.Errorf("cols is required")
	}

	rows, ok := params["rows"].(float64)
	if !ok {
		return nil, fmt.Errorf("rows is required")
	}

	err := p.manager.Resize(sessionID, int(cols), int(rows))
	if err != nil {
		return nil, err
	}

	return &types.Result{
		Success: true,
		Data:    map[string]interface{}{"success": true},
	}, nil
}

func (p *Provider) listSessions() (*types.Result, error) {
	sessions := p.manager.ListSessions()

	return &types.Result{
		Success: true,
		Data: map[string]interface{}{
			"sessions": sessions,
			"count":    len(sessions),
		},
	}, nil
}

func (p *Provider) getSession(params map[string]interface{}) (*types.Result, error) {
	sessionID, ok := params["session_id"].(string)
	if !ok {
		return nil, fmt.Errorf("session_id is required")
	}

	session, err := p.manager.GetSession(sessionID)
	if err != nil {
		return nil, err
	}

	sessionData2 := map[string]interface{}{
		"id":          session.ID,
		"shell":       session.Shell,
		"working_dir": session.WorkingDir,
		"cols":        session.Cols,
		"rows":        session.Rows,
		"started_at":  session.StartedAt,
		"active":      session.Active,
	}

	return &types.Result{
		Success: true,
		Data:    sessionData2,
	}, nil
}

func (p *Provider) kill(params map[string]interface{}) (*types.Result, error) {
	sessionID, ok := params["session_id"].(string)
	if !ok {
		return nil, fmt.Errorf("session_id is required")
	}

	err := p.manager.Kill(sessionID)
	if err != nil {
		return nil, err
	}

	return &types.Result{
		Success: true,
		Data:    map[string]interface{}{"success": true},
	}, nil
}
