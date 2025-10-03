/**
 * Dynamic Renderer Component
 * Renders AI-generated applications on the fly
 */

import React, { useState, useCallback } from 'react';
import './DynamicRenderer.css';

// ============================================================================
// Type Definitions
// ============================================================================

interface UIComponent {
  type: string;
  id: string;
  props: Record<string, any>;
  children?: UIComponent[];
  on_event?: Record<string, string>; // event name -> tool_id
}

interface UISpec {
  type: string;
  title: string;
  layout: string;
  components: UIComponent[];
  style?: Record<string, any>;
}

// ============================================================================
// Component State Manager
// ============================================================================

class ComponentState {
  private state: Map<string, any> = new Map();
  private listeners: Map<string, Set<(value: any) => void>> = new Map();

  get(key: string, defaultValue: any = null): any {
    return this.state.get(key) ?? defaultValue;
  }

  set(key: string, value: any): void {
    this.state.set(key, value);
    // Notify listeners
    const listeners = this.listeners.get(key);
    if (listeners) {
      listeners.forEach(listener => listener(value));
    }
  }

  subscribe(key: string, listener: (value: any) => void): () => void {
    if (!this.listeners.has(key)) {
      this.listeners.set(key, new Set());
    }
    this.listeners.get(key)!.add(listener);
    
    // Return unsubscribe function
    return () => {
      const listeners = this.listeners.get(key);
      if (listeners) {
        listeners.delete(listener);
      }
    };
  }

  clear(): void {
    this.state.clear();
    this.listeners.clear();
  }
}

// ============================================================================
// Tool Execution
// ============================================================================

class ToolExecutor {
  private componentState: ComponentState;

  constructor(componentState: ComponentState) {
    this.componentState = componentState;
  }

  async execute(toolId: string, params: Record<string, any> = {}): Promise<any> {
    console.log(`[ToolExecutor] Executing tool: ${toolId}`, params);

    // Parse tool ID (category.action)
    const [category, action] = toolId.split('.');

    switch (category) {
      case 'calc':
        return this.executeCalcTool(action, params);
      case 'ui':
        return this.executeUITool(action, params);
      case 'system':
        return this.executeSystemTool(action, params);
      case 'app':
        return await this.executeAppTool(action, params);
      default:
        console.warn(`Unknown tool category: ${category}`);
        return null;
    }
  }

  private executeCalcTool(action: string, params: Record<string, any>): any {
    const a = params.a || 0;
    const b = params.b || 0;

    switch (action) {
      case 'add':
        return a + b;
      case 'subtract':
        return a - b;
      case 'multiply':
        return a * b;
      case 'divide':
        return b !== 0 ? a / b : 'Error';
      case 'append_digit':
        const current = this.componentState.get('display', '0');
        const digit = params.digit || '';
        const newValue = current === '0' ? digit : current + digit;
        this.componentState.set('display', newValue);
        return newValue;
      case 'clear':
        this.componentState.set('display', '0');
        return '0';
      case 'evaluate':
        try {
          const expression = this.componentState.get('display', '0');
          // Simple eval (in production, use a proper math parser!)
          const result = eval(expression.replace('×', '*').replace('÷', '/').replace('−', '-'));
          this.componentState.set('display', String(result));
          return result;
        } catch {
          this.componentState.set('display', 'Error');
          return 'Error';
        }
      default:
        return null;
    }
  }

  private executeUITool(action: string, params: Record<string, any>): any {
    switch (action) {
      case 'set_state':
        this.componentState.set(params.key, params.value);
        return params.value;
      case 'get_state':
        return this.componentState.get(params.key);
      case 'add_todo':
        const todos = this.componentState.get('todos', []);
        const newTask = this.componentState.get('task-input', '');
        if (newTask.trim()) {
          todos.push({ id: Date.now(), text: newTask, done: false });
          this.componentState.set('todos', [...todos]);
          this.componentState.set('task-input', '');
        }
        return todos;
      default:
        return null;
    }
  }

  private executeSystemTool(action: string, params: Record<string, any>): any {
    switch (action) {
      case 'alert':
        alert(params.message);
        return true;
      case 'log':
        console.log(`[System] ${params.message}`);
        return true;
      default:
        return null;
    }
  }

  private async executeAppTool(action: string, params: Record<string, any>): Promise<any> {
    switch (action) {
      case 'spawn':
        // Spawn a new app via WebSocket for real-time updates
        console.log(`[App] Spawning new app: ${params.request}`);
        
        // Connect to WebSocket
        const ws = new WebSocket('ws://localhost:8000/stream');
        
        return new Promise((resolve, reject) => {
          ws.onopen = () => {
            ws.send(JSON.stringify({
              type: 'generate_ui',
              message: params.request,
              context: { parent_app_id: this.componentState.get('app_id') }
            }));
          };
          
          ws.onmessage = (event) => {
            const data = JSON.parse(event.data);
            
            if (data.type === 'thought') {
              console.log(`[App.spawn] 💭 ${data.content}`);
            } else if (data.type === 'ui_generated') {
              console.log(`[App.spawn] ✅ Spawned: ${data.ui_spec.title}`);
              // Notify parent component to render new app
              window.postMessage({ 
                type: 'spawn_app', 
                app_id: data.app_id,
                ui_spec: data.ui_spec 
              }, '*');
              resolve(data.ui_spec);
            } else if (data.type === 'error') {
              console.error(`[App.spawn] ❌ ${data.message}`);
              reject(new Error(data.message));
            } else if (data.type === 'complete') {
              ws.close();
            }
          };
          
          ws.onerror = (err) => {
            console.error('[App.spawn] WebSocket error:', err);
            reject(err);
          };
        });
        
      case 'close':
        console.log('[App] Closing current app');
        // Notify parent to close this app
        window.postMessage({ type: 'close_app' }, '*');
        return true;
        
      case 'list':
        console.log('[App] Listing apps');
        const response = await fetch('http://localhost:8000/apps');
        const data = await response.json();
        return data.apps;
        
      default:
        return null;
    }
  }
}

// ============================================================================
// Component Renderers
// ============================================================================

interface RendererProps {
  component: UIComponent;
  state: ComponentState;
  executor: ToolExecutor;
}

const ComponentRenderer: React.FC<RendererProps> = ({ component, state, executor }) => {
  const [, forceUpdate] = useState({});

  // Subscribe to state changes for this component
  React.useEffect(() => {
    if (component.id) {
      const unsubscribe = state.subscribe(component.id, () => {
        forceUpdate({});
      });
      return unsubscribe;
    }
  }, [component.id, state]);

  const handleEvent = useCallback((eventName: string, eventData?: any) => {
    const toolId = component.on_event?.[eventName];
    if (toolId) {
      // Extract params from event or component
      const params = {
        ...eventData,
        componentId: component.id,
        digit: component.props.text, // For calculator buttons
      };
      executor.execute(toolId, params);
    }
  }, [component, executor]);

  // Render based on component type
  switch (component.type) {
    case 'button':
      return (
        <button
          className="dynamic-button"
          onClick={() => handleEvent('click')}
          style={component.props.style}
        >
          {component.props.text || 'Button'}
        </button>
      );

    case 'input':
      const value = state.get(component.id, component.props.value || '');
      return (
        <input
          className="dynamic-input"
          type={component.props.type || 'text'}
          placeholder={component.props.placeholder}
          value={value}
          readOnly={component.props.readonly}
          onChange={(e) => state.set(component.id, e.target.value)}
          style={component.props.style}
        />
      );

    case 'text':
      const variant = component.props.variant || 'body';
      const Tag = variant === 'h1' ? 'h1' : variant === 'h2' ? 'h2' : 'p';
      return (
        <Tag className={`dynamic-text dynamic-text-${variant}`} style={component.props.style}>
          {component.props.content}
        </Tag>
      );

    case 'container':
      return (
        <div
          className={`dynamic-container dynamic-container-${component.props.layout || 'vertical'}`}
          style={{ gap: `${component.props.gap || 8}px`, ...component.props.style }}
        >
          {component.children?.map((child, idx) => (
            <ComponentRenderer
              key={`${child.id}-${idx}`}
              component={child}
              state={state}
              executor={executor}
            />
          ))}
        </div>
      );

    case 'grid':
      return (
        <div
          className="dynamic-grid"
          style={{
            gridTemplateColumns: `repeat(${component.props.columns || 3}, 1fr)`,
            gap: `${component.props.gap || 8}px`,
            ...component.props.style,
          }}
        >
          {component.children?.map((child, idx) => (
            <ComponentRenderer
              key={`${child.id}-${idx}`}
              component={child}
              state={state}
              executor={executor}
            />
          ))}
        </div>
      );

    default:
      return (
        <div className="dynamic-unknown">
          Unknown component: {component.type}
        </div>
      );
  }
};

// ============================================================================
// Main DynamicRenderer Component
// ============================================================================

const DynamicRenderer: React.FC = () => {
  const [uiSpec, setUiSpec] = useState<UISpec | null>(null);
  const [componentState] = useState(() => new ComponentState());
  const [toolExecutor] = useState(() => new ToolExecutor(componentState));
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadUISpec = useCallback(async (request: string) => {
    setIsLoading(true);
    setError(null);
    
    try {
      // Connect to WebSocket for real-time streaming
      const ws = new WebSocket('ws://localhost:8000/stream');
      
      ws.onopen = () => {
        console.log('[DynamicRenderer] WebSocket connected');
        // Request UI generation via WebSocket
        ws.send(JSON.stringify({
          type: 'generate_ui',
          message: request,
          context: {}
        }));
      };
      
      ws.onmessage = (event) => {
        const data = JSON.parse(event.data);
        console.log('[DynamicRenderer] Received:', data.type, data);
        
        switch (data.type) {
          case 'generation_start':
            console.log('[DynamicRenderer] 🎨 Generation started:', data.message);
            break;
            
          case 'thought':
            console.log('[DynamicRenderer] 💭 Thought:', data.content);
            // Could show these in UI as status updates!
            break;
            
          case 'ui_generated':
            console.log('[DynamicRenderer] ✅ UI generated:', data.ui_spec.title);
            setUiSpec(data.ui_spec);
            componentState.clear();
            break;
            
          case 'complete':
            console.log('[DynamicRenderer] ✨ Generation complete');
            setIsLoading(false);
            ws.close();
            break;
            
          case 'error':
            console.error('[DynamicRenderer] ❌ Error:', data.message);
            setError(data.message);
            setIsLoading(false);
            ws.close();
            break;
        }
      };
      
      ws.onerror = (err) => {
        console.error('[DynamicRenderer] WebSocket error:', err);
        setError('WebSocket connection failed');
        setIsLoading(false);
      };
      
      ws.onclose = () => {
        console.log('[DynamicRenderer] WebSocket closed');
      };
      
    } catch (err) {
      console.error('[DynamicRenderer] Error loading UI:', err);
      setError(err instanceof Error ? err.message : 'Failed to load UI');
      setIsLoading(false);
    }
  }, [componentState]);

  // Example: Load calculator on mount (for testing)
  React.useEffect(() => {
    loadUISpec('create a calculator');
  }, [loadUISpec]);

  return (
    <div className="dynamic-renderer">
      <div className="renderer-header">
        <h3>⚡ App Renderer</h3>
        <span className={`renderer-status ${uiSpec ? 'active' : ''}`}>
          {isLoading ? 'Loading...' : uiSpec ? 'Active' : 'Ready'}
        </span>
      </div>

      <div className="renderer-canvas">
        {error && (
          <div className="renderer-error">
            <strong>Error:</strong> {error}
          </div>
        )}

        {!uiSpec && !isLoading && !error && (
          <div className="placeholder">
            <span className="placeholder-icon">🎨</span>
            <h2>Dynamic App Canvas</h2>
            <p>AI-generated applications will render here in real-time</p>
            <div className="example-apps">
              <button className="app-card" onClick={() => loadUISpec('create a calculator')}>
                📱 Calculator
              </button>
              <button className="app-card" onClick={() => loadUISpec('create a todo app')}>
                📝 Todo App
              </button>
              <button className="app-card" onClick={() => loadUISpec('create a counter')}>
                🔢 Counter
              </button>
              <button className="app-card" onClick={() => loadUISpec('create a settings page')}>
                ⚙️ Settings
              </button>
            </div>
          </div>
        )}

        {uiSpec && (
          <div className="rendered-app" style={uiSpec.style}>
            <div className="app-header">
              <h2>{uiSpec.title}</h2>
            </div>
            <div className={`app-content app-layout-${uiSpec.layout}`}>
              {uiSpec.components.map((component, idx) => (
                <ComponentRenderer
                  key={`${component.id}-${idx}`}
                  component={component}
                  state={componentState}
                  executor={toolExecutor}
                />
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default DynamicRenderer;

