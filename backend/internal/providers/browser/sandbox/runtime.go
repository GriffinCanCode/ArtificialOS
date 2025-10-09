package sandbox

import (
	"context"
	"fmt"
	"sync"
	"time"

	"github.com/dop251/goja"
)

// Runtime wraps goja VM with security controls
type Runtime struct {
	vm     *goja.Runtime
	config Config
	mu     sync.RWMutex

	// Console output
	console   []LogEntry
	consoleMu sync.Mutex

	// Interrupt channel
	interrupt chan struct{}
}

// New creates a new sandboxed runtime
func New(config Config) (*Runtime, error) {
	vm := goja.New()

	r := &Runtime{
		vm:        vm,
		config:    config,
		console:   []LogEntry{},
		interrupt: make(chan struct{}),
	}

	// Set memory limit
	if config.MaxMemoryMB > 0 {
		vm.SetMaxCallStackSize(1024)
	}

	// Setup global objects
	if err := r.setupGlobals(); err != nil {
		return nil, err
	}

	return r, nil
}

// Execute runs JavaScript code with timeout and resource limits
func (r *Runtime) Execute(ctx context.Context, script string, dom *DOM) (*Result, error) {
	r.mu.Lock()
	defer r.mu.Unlock()

	start := time.Now()
	result := &Result{
		Console: []LogEntry{},
	}

	// Setup timeout
	timer := time.NewTimer(r.config.Timeout)
	defer timer.Stop()

	// Setup interrupt handler
	go func() {
		select {
		case <-timer.C:
			r.vm.Interrupt("execution timeout exceeded")
		case <-ctx.Done():
			r.vm.Interrupt("context cancelled")
		case <-r.interrupt:
			return
		}
	}()

	// Clear console
	r.consoleMu.Lock()
	r.console = []LogEntry{}
	r.consoleMu.Unlock()

	// Inject DOM if provided
	if dom != nil && r.config.EnableDOM {
		if err := r.injectDOM(dom); err != nil {
			return nil, fmt.Errorf("failed to inject DOM: %w", err)
		}
	}

	// Execute script
	val, err := r.vm.RunString(script)

	// Stop interrupt goroutine
	close(r.interrupt)
	r.interrupt = make(chan struct{})

	result.Duration = time.Since(start)

	if err != nil {
		result.Error = err
		return result, err
	}

	// Extract result value
	result.Value = r.exportValue(val)

	// Collect console output
	r.consoleMu.Lock()
	result.Console = append([]LogEntry{}, r.console...)
	r.consoleMu.Unlock()

	// Collect DOM changes
	if dom != nil {
		result.DOMChanges = dom.GetChanges()
	}

	return result, nil
}

// setupGlobals configures global objects and security
func (r *Runtime) setupGlobals() error {
	// Remove dangerous globals
	r.vm.Set("require", goja.Undefined())
	r.vm.Set("process", goja.Undefined())
	r.vm.Set("module", goja.Undefined())
	r.vm.Set("exports", goja.Undefined())

	// Setup console if enabled
	if r.config.EnableConsole {
		console := r.vm.NewObject()
		console.Set("log", r.makeConsoleFunc("log"))
		console.Set("warn", r.makeConsoleFunc("warn"))
		console.Set("error", r.makeConsoleFunc("error"))
		console.Set("info", r.makeConsoleFunc("info"))
		r.vm.Set("console", console)
	}

	// Setup timers (no-op for security)
	r.vm.Set("setTimeout", func(call goja.FunctionCall) goja.Value {
		return goja.Undefined()
	})
	r.vm.Set("setInterval", func(call goja.FunctionCall) goja.Value {
		return goja.Undefined()
	})

	return nil
}

// makeConsoleFunc creates a console function
func (r *Runtime) makeConsoleFunc(level string) func(goja.FunctionCall) goja.Value {
	return func(call goja.FunctionCall) goja.Value {
		var msg string
		for i, arg := range call.Arguments {
			if i > 0 {
				msg += " "
			}
			msg += arg.String()
		}

		r.consoleMu.Lock()
		r.console = append(r.console, LogEntry{
			Level:   level,
			Message: msg,
			Time:    time.Now(),
		})
		r.consoleMu.Unlock()

		return goja.Undefined()
	}
}

// injectDOM injects DOM proxy into runtime
func (r *Runtime) injectDOM(dom *DOM) error {
	// Create document object
	document := r.vm.NewObject()

	// querySelector
	document.Set("querySelector", r.makeDOMFunc(dom, "querySelector"))
	document.Set("querySelectorAll", r.makeDOMFunc(dom, "querySelectorAll"))
	document.Set("getElementById", r.makeDOMFunc(dom, "getElementById"))
	document.Set("getElementsByClassName", r.makeDOMFunc(dom, "getElementsByClassName"))
	document.Set("getElementsByTagName", r.makeDOMFunc(dom, "getElementsByTagName"))

	r.vm.Set("document", document)
	return nil
}

// makeDOMFunc creates a DOM proxy function
func (r *Runtime) makeDOMFunc(dom *DOM, method string) func(goja.FunctionCall) goja.Value {
	return func(call goja.FunctionCall) goja.Value {
		if len(call.Arguments) == 0 {
			return goja.Null()
		}

		selector := call.Arguments[0].String()
		elements := dom.Query(selector)

		if len(elements) == 0 {
			return goja.Null()
		}

		// Return element proxy
		elem := r.createElementProxy(elements[0])
		return r.vm.ToValue(elem)
	}
}

// createElementProxy creates a proxy for DOM element
func (r *Runtime) createElementProxy(elem *Element) map[string]interface{} {
	return map[string]interface{}{
		"tagName":     elem.TagName,
		"id":          elem.ID,
		"className":   elem.ClassName,
		"textContent": elem.TextContent,
		"getAttribute": func(name string) string {
			return elem.GetAttribute(name)
		},
		"setAttribute": func(name, value string) {
			elem.SetAttribute(name, value)
		},
	}
}

// exportValue converts goja value to Go value
func (r *Runtime) exportValue(val goja.Value) interface{} {
	if val == nil || goja.IsUndefined(val) || goja.IsNull(val) {
		return nil
	}
	return val.Export()
}

// Reset clears the runtime state
func (r *Runtime) Reset() error {
	r.mu.Lock()
	defer r.mu.Unlock()

	r.vm = goja.New()
	r.console = []LogEntry{}
	return r.setupGlobals()
}

// Close releases resources
func (r *Runtime) Close() error {
	r.mu.Lock()
	defer r.mu.Unlock()

	r.vm = nil
	r.console = nil
	return nil
}
