/*
Package sandbox provides JavaScript execution sandboxing for the browser.

# Overview

The sandbox system enables safe execution of untrusted JavaScript code within
isolated runtimes using the goja JavaScript engine. Each sandbox has:

  - Memory limits (configurable heap size)
  - CPU limits (execution timeout, interrupt polling)
  - API restrictions (disabled Node.js, filesystem, network)
  - DOM proxy for safe document manipulation
  - Secure communication bridge for controlled host access

# Architecture

The sandbox operates in multiple layers:

 1. Runtime: goja VM with isolated global scope
 2. DOM Proxy: Lightweight document object model emulation
 3. Bridge: Secure message passing to host environment
 4. Resource Limiter: Memory and CPU usage monitoring

# Security Model

Sandboxed code cannot:
  - Access filesystem or network directly
  - Execute native code or spawn processes
  - Break out of the VM through prototype pollution
  - Consume excessive memory or CPU time

All host interactions go through the secure bridge with validation.

# Usage Example

	sandbox := New(Config{
		MaxMemoryMB: 50,
		Timeout:     5 * time.Second,
	})

	result, err := sandbox.Execute(ctx, script, dom)
	if err != nil {
		log.Error("Execution failed:", err)
	}

# Performance

  - ~1-2ms startup overhead per sandbox
  - ~50MB memory footprint per instance
  - Sub-millisecond bridge calls
  - Automatic garbage collection

# Integration

The sandbox integrates with:
  - Browser provider for page scripts
  - Kernel permissions for resource limits
  - Monitoring for execution metrics
*/
package sandbox
