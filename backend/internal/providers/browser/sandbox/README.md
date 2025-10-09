# Browser JavaScript Sandbox

Secure JavaScript execution for untrusted web content.

## Architecture

```
┌─────────────────────────────────────────────┐
│         JavaScript Sandbox System           │
├─────────────────────────────────────────────┤
│                                             │
│  ┌─────────────┐      ┌────────────────┐  │
│  │   Pool      │◄─────┤   Runtime      │  │
│  │             │      │   (goja VM)    │  │
│  │ 4 VMs       │      │                │  │
│  │ Pre-warmed  │      │ - Isolated     │  │
│  │ Auto-reset  │      │ - Timeout      │  │
│  └─────────────┘      │ - Memory limit │  │
│                       └────────────────┘  │
│                                             │
│  ┌─────────────┐      ┌────────────────┐  │
│  │   DOM       │      │    Bridge      │  │
│  │   Proxy     │      │  (Host API)    │  │
│  │             │      │                │  │
│  │ - Query     │      │ - Secure calls │  │
│  │ - Manipulate│      │ - Validation   │  │
│  └─────────────┘      └────────────────┘  │
│                                             │
└─────────────────────────────────────────────┘
```

## Security Model

### Isolation

Each sandbox runs in a separate goja VM with:
- Isolated global scope
- No access to Go runtime
- No filesystem or network access
- No native code execution

### Resource Limits

```go
Config{
    MaxMemoryMB: 50,        // Heap limit
    Timeout: 5 * time.Second, // Max execution time
}
```

### Disabled APIs

Removed dangerous globals:
- `require()` - No module loading
- `process` - No process access
- `module/exports` - No CommonJS
- `setTimeout/setInterval` - No timers (prevent infinite loops)

### Safe APIs

Available to scripts:
- `console.log/warn/error` - Logging (captured)
- `document` - DOM proxy (limited API)
- Math, JSON, String, Array - Standard built-ins

## Usage

### Basic Execution

```go
sandbox, _ := sandbox.New(sandbox.DefaultConfig())
result, err := sandbox.Execute(ctx, `
  console.log("Hello from sandbox");
  return 42;
`, nil)
```

### With DOM

```go
dom := sandbox.NewDOM()
result, _ := sandbox.Execute(ctx, `
  const elem = document.querySelector('#my-id');
  elem.setAttribute('class', 'active');
`, dom)
```

### Pool for Performance

```go
pool, _ := sandbox.NewPool(config, 4)
result, _ := pool.Execute(ctx, script, dom)
```

## Performance

- **Cold start**: ~1-2ms (VM creation + setup)
- **Warm start**: <100μs (pool reuse)
- **Memory**: ~50MB per VM
- **Execution**: Depends on script complexity

Pool eliminates cold starts by pre-warming VMs.

## Files

- `doc.go` - Package documentation
- `types.go` - Core types and interfaces
- `runtime.go` - goja VM wrapper with security
- `dom.go` - Lightweight DOM proxy
- `pool.go` - VM pool for performance

## Testing

Run sandbox security tests:

```bash
cd backend
go test ./internal/providers/browser/sandbox/... -v
```

## Future Enhancements

- [ ] Fetch API through secure bridge
- [ ] Full DOM API (createElement, etc.)
- [ ] Event handling system
- [ ] WebAssembly support
- [ ] Streaming execution for long scripts
- [ ] Memory profiling and optimization

