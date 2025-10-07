package kernel

import (
	"context"
	"encoding/json"
	"fmt"
	"time"

	pb "github.com/GriffinCanCode/AgentOS/backend/proto/kernel"
)

// Signal constants matching UNIX signals
const (
	SIGHUP    = 1
	SIGINT    = 2
	SIGQUIT   = 3
	SIGILL    = 4
	SIGTRAP   = 5
	SIGABRT   = 6
	SIGBUS    = 7
	SIGFPE    = 8
	SIGKILL   = 9
	SIGUSR1   = 10
	SIGSEGV   = 11
	SIGUSR2   = 12
	SIGPIPE   = 13
	SIGALRM   = 14
	SIGTERM   = 15
	SIGCHLD   = 17
	SIGCONT   = 18
	SIGSTOP   = 19
	SIGTSTP   = 20
	SIGTTIN   = 21
	SIGTTOU   = 22
	SIGURG    = 23
	SIGXCPU   = 24
	SIGXFSZ   = 25
	SIGVTALRM = 26
	SIGPROF   = 27
	SIGWINCH  = 28
	SIGIO     = 29
	SIGPWR    = 30
	SIGSYS    = 31
	SIGRTMIN  = 34
	SIGRTMAX  = 63
)

// SignalStats represents signal statistics
type SignalStats struct {
	TotalSignalsSent      uint64 `json:"total_signals_sent"`
	TotalSignalsDelivered uint64 `json:"total_signals_delivered"`
	TotalSignalsBlocked   uint64 `json:"total_signals_blocked"`
	TotalSignalsQueued    uint64 `json:"total_signals_queued"`
	PendingSignals        uint64 `json:"pending_signals"`
	HandlersRegistered    uint64 `json:"handlers_registered"`
}

// PendingSignal represents a pending signal
type PendingSignal struct {
	Signal    uint32 `json:"signal"`
	SenderPid uint32 `json:"sender_pid"`
	Timestamp uint64 `json:"timestamp"`
}

// ProcessSignalState represents complete signal state for a process
type ProcessSignalState struct {
	Pid                uint32            `json:"pid"`
	PendingSignals     []PendingSignal   `json:"pending_signals,omitempty"`
	BlockedSignals     []uint32          `json:"blocked_signals,omitempty"`
	RegisteredHandlers map[uint32]uint64 `json:"handlers,omitempty"`
}

// buildSignalSyscall builds signal syscall requests
func (k *KernelClient) buildSignalSyscall(req *pb.SyscallRequest, syscallType string, params map[string]interface{}) {
	switch syscallType {
	case "send_signal":
		targetPid, _ := params["target_pid"].(uint32)
		signal, _ := params["signal"].(uint32)
		req.Syscall = &pb.SyscallRequest_SendSignal{
			SendSignal: &pb.SendSignalCall{TargetPid: targetPid, Signal: signal},
		}
	case "register_signal_handler":
		signal, _ := params["signal"].(uint32)
		handlerID, _ := params["handler_id"].(uint32)
		req.Syscall = &pb.SyscallRequest_RegisterSignalHandler{
			RegisterSignalHandler: &pb.RegisterSignalHandlerCall{Signal: signal, HandlerId: handlerID},
		}
	case "block_signal":
		signal, _ := params["signal"].(uint32)
		req.Syscall = &pb.SyscallRequest_BlockSignal{
			BlockSignal: &pb.BlockSignalCall{Signal: signal},
		}
	case "unblock_signal":
		signal, _ := params["signal"].(uint32)
		req.Syscall = &pb.SyscallRequest_UnblockSignal{
			UnblockSignal: &pb.UnblockSignalCall{Signal: signal},
		}
	case "get_pending_signals":
		req.Syscall = &pb.SyscallRequest_GetPendingSignals{
			GetPendingSignals: &pb.GetPendingSignalsCall{},
		}
	case "get_signal_stats":
		req.Syscall = &pb.SyscallRequest_GetSignalStats{
			GetSignalStats: &pb.GetSignalStatsCall{},
		}
	case "get_signal_state":
		if targetPid, ok := params["target_pid"].(uint32); ok {
			req.Syscall = &pb.SyscallRequest_GetSignalState{
				GetSignalState: &pb.GetSignalStateCall{TargetPid: &targetPid},
			}
		} else {
			req.Syscall = &pb.SyscallRequest_GetSignalState{
				GetSignalState: &pb.GetSignalStateCall{},
			}
		}
	case "wait_for_signal":
		signals, _ := params["signals"].([]uint32)
		if timeoutMs, ok := params["timeout_ms"].(uint64); ok {
			req.Syscall = &pb.SyscallRequest_WaitForSignal{
				WaitForSignal: &pb.WaitForSignalCall{Signals: signals, TimeoutMs: &timeoutMs},
			}
		} else {
			req.Syscall = &pb.SyscallRequest_WaitForSignal{
				WaitForSignal: &pb.WaitForSignalCall{Signals: signals},
			}
		}
	}
}

// SendSignal sends a signal to a target process
func (k *KernelClient) SendSignal(ctx context.Context, pid uint32, targetPid uint32, signal uint32) error {
	req := &pb.SyscallRequest{
		Pid: pid,
		Syscall: &pb.SyscallRequest_SendSignal{
			SendSignal: &pb.SendSignalCall{
				TargetPid: targetPid,
				Signal:    signal,
			},
		},
	}

	resp, err := k.ExecuteSyscallRaw(ctx, req)
	if err != nil {
		return fmt.Errorf("failed to send signal: %w", err)
	}

	return extractError(resp)
}

// RegisterSignalHandler registers a signal handler for a process
func (k *KernelClient) RegisterSignalHandler(ctx context.Context, pid uint32, signal uint32, handlerID uint32) error {
	req := &pb.SyscallRequest{
		Pid: pid,
		Syscall: &pb.SyscallRequest_RegisterSignalHandler{
			RegisterSignalHandler: &pb.RegisterSignalHandlerCall{
				Signal:    signal,
				HandlerId: handlerID,
			},
		},
	}

	resp, err := k.ExecuteSyscallRaw(ctx, req)
	if err != nil {
		return fmt.Errorf("failed to register signal handler: %w", err)
	}

	return extractError(resp)
}

// BlockSignal blocks a signal from being delivered
func (k *KernelClient) BlockSignal(ctx context.Context, pid uint32, signal uint32) error {
	req := &pb.SyscallRequest{
		Pid: pid,
		Syscall: &pb.SyscallRequest_BlockSignal{
			BlockSignal: &pb.BlockSignalCall{
				Signal: signal,
			},
		},
	}

	resp, err := k.ExecuteSyscallRaw(ctx, req)
	if err != nil {
		return fmt.Errorf("failed to block signal: %w", err)
	}

	return extractError(resp)
}

// UnblockSignal unblocks a signal
func (k *KernelClient) UnblockSignal(ctx context.Context, pid uint32, signal uint32) error {
	req := &pb.SyscallRequest{
		Pid: pid,
		Syscall: &pb.SyscallRequest_UnblockSignal{
			UnblockSignal: &pb.UnblockSignalCall{
				Signal: signal,
			},
		},
	}

	resp, err := k.ExecuteSyscallRaw(ctx, req)
	if err != nil {
		return fmt.Errorf("failed to unblock signal: %w", err)
	}

	return extractError(resp)
}

// GetPendingSignals retrieves all pending signals for a process
func (k *KernelClient) GetPendingSignals(ctx context.Context, pid uint32) ([]uint32, error) {
	req := &pb.SyscallRequest{
		Pid: pid,
		Syscall: &pb.SyscallRequest_GetPendingSignals{
			GetPendingSignals: &pb.GetPendingSignalsCall{},
		},
	}

	resp, err := k.ExecuteSyscallRaw(ctx, req)
	if err != nil {
		return nil, fmt.Errorf("failed to get pending signals: %w", err)
	}

	if err := extractError(resp); err != nil {
		return nil, err
	}

	// Parse response data
	var signals []uint32
	if data := extractData(resp); data != nil {
		if err := json.Unmarshal(data, &signals); err != nil {
			return nil, fmt.Errorf("failed to parse pending signals: %w", err)
		}
	}

	return signals, nil
}

// GetSignalStats retrieves signal statistics for the calling process
func (k *KernelClient) GetSignalStats(ctx context.Context, pid uint32) (*SignalStats, error) {
	req := &pb.SyscallRequest{
		Pid: pid,
		Syscall: &pb.SyscallRequest_GetSignalStats{
			GetSignalStats: &pb.GetSignalStatsCall{},
		},
	}

	resp, err := k.ExecuteSyscallRaw(ctx, req)
	if err != nil {
		return nil, fmt.Errorf("failed to get signal stats: %w", err)
	}

	if err := extractError(resp); err != nil {
		return nil, err
	}

	// Parse response data
	var stats SignalStats
	if data := extractData(resp); data != nil {
		if err := json.Unmarshal(data, &stats); err != nil {
			return nil, fmt.Errorf("failed to parse signal stats: %w", err)
		}
	}

	return &stats, nil
}

// GetSignalState retrieves complete signal state for a process
func (k *KernelClient) GetSignalState(ctx context.Context, pid uint32, targetPid *uint32) (*ProcessSignalState, error) {
	req := &pb.SyscallRequest{
		Pid: pid,
		Syscall: &pb.SyscallRequest_GetSignalState{
			GetSignalState: &pb.GetSignalStateCall{
				TargetPid: targetPid,
			},
		},
	}

	resp, err := k.ExecuteSyscallRaw(ctx, req)
	if err != nil {
		return nil, fmt.Errorf("failed to get signal state: %w", err)
	}

	if err := extractError(resp); err != nil {
		return nil, err
	}

	// Parse response data
	var state ProcessSignalState
	if data := extractData(resp); data != nil {
		if err := json.Unmarshal(data, &state); err != nil {
			return nil, fmt.Errorf("failed to parse signal state: %w", err)
		}
	}

	return &state, nil
}

// WaitForSignal waits for specific signals with optional timeout
func (k *KernelClient) WaitForSignal(ctx context.Context, pid uint32, signals []uint32, timeout *time.Duration) (uint32, error) {
	var timeoutMs *uint64
	if timeout != nil {
		ms := uint64(timeout.Milliseconds())
		timeoutMs = &ms
	}

	req := &pb.SyscallRequest{
		Pid: pid,
		Syscall: &pb.SyscallRequest_WaitForSignal{
			WaitForSignal: &pb.WaitForSignalCall{
				Signals:   signals,
				TimeoutMs: timeoutMs,
			},
		},
	}

	resp, err := k.ExecuteSyscallRaw(ctx, req)
	if err != nil {
		return 0, fmt.Errorf("failed to wait for signal: %w", err)
	}

	if err := extractError(resp); err != nil {
		return 0, err
	}

	// Parse response data - should return the signal number that was received
	var signal uint32
	if data := extractData(resp); data != nil {
		if err := json.Unmarshal(data, &signal); err != nil {
			return 0, fmt.Errorf("failed to parse signal: %w", err)
		}
	}

	return signal, nil
}
