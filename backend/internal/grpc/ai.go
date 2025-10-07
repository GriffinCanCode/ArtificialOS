package grpc

import (
	"context"
	"fmt"
	"io"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/keepalive"

	"github.com/GriffinCanCode/AgentOS/backend/internal/infrastructure/resilience"
	pb "github.com/GriffinCanCode/AgentOS/backend/proto/ai"
)

// AIClient wraps gRPC client for AI service communication with circuit breaker
type AIClient struct {
	conn    *grpc.ClientConn
	client  pb.AIServiceClient
	addr    string
	breaker *resilience.Breaker
}

// NewAIClient creates a new AI client with proper connection management
func NewAIClient(addr string) (*AIClient, error) {
	// Configure connection options for production use
	opts := []grpc.DialOption{
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		// Configure keepalive to detect broken connections
		grpc.WithKeepaliveParams(keepalive.ClientParameters{
			Time:                10 * time.Second, // Send pings every 10 seconds
			Timeout:             3 * time.Second,  // Wait 3 seconds for ping ack
			PermitWithoutStream: true,             // Allow pings without active streams
		}),
		// Set larger message size limits for AI responses (can be large)
		grpc.WithDefaultCallOptions(
			grpc.MaxCallRecvMsgSize(50*1024*1024), // 50MB receive limit for large AI responses
			grpc.MaxCallSendMsgSize(10*1024*1024), // 10MB send limit
		),
	}

	// Dial without WithBlock() - it's deprecated and problematic
	conn, err := grpc.Dial(addr, opts...)
	if err != nil {
		return nil, fmt.Errorf("failed to dial AI service: %w", err)
	}

	// Create circuit breaker for AI service calls
	breaker := resilience.New("ai-service", resilience.Settings{
		MaxRequests: 2,
		Interval:    60 * time.Second,
		Timeout:     30 * time.Second,
		ReadyToTrip: func(counts resilience.Counts) bool {
			// AI service is more sensitive - trip after 3 consecutive failures
			return counts.ConsecutiveFailures >= 3
		},
	})

	return &AIClient{
		conn:    conn,
		client:  pb.NewAIServiceClient(conn),
		addr:    addr,
		breaker: breaker,
	}, nil
}

// Close closes the connection
func (a *AIClient) Close() error {
	if a.conn != nil {
		return a.conn.Close()
	}
	return nil
}

// GenerateUI generates a UI specification with circuit breaker protection
func (a *AIClient) GenerateUI(ctx context.Context, message string, contextMap map[string]string, parentID *string) (*pb.UIResponse, error) {
	// Use provided context with timeout
	ctx, cancel := context.WithTimeout(ctx, 60*time.Second)
	defer cancel()

	req := &pb.UIRequest{
		Message: message,
		Context: contextMap,
	}
	if parentID != nil {
		req.ParentId = parentID
	}

	result, err := a.breaker.Execute(func() (interface{}, error) {
		return a.client.GenerateUI(ctx, req)
	})

	if err == resilience.ErrCircuitOpen {
		return nil, fmt.Errorf("AI service unavailable: circuit breaker open")
	}

	if err != nil {
		return nil, err
	}

	return result.(*pb.UIResponse), nil
}

// StreamUI streams UI generation with real-time updates
func (a *AIClient) StreamUI(ctx context.Context, message string, contextMap map[string]string, parentID *string) (pb.AIService_StreamUIClient, error) {
	req := &pb.UIRequest{
		Message: message,
		Context: contextMap,
	}
	if parentID != nil {
		req.ParentId = parentID
	}

	return a.client.StreamUI(ctx, req)
}

// StreamChat streams chat response
func (a *AIClient) StreamChat(ctx context.Context, message string, contextMap map[string]string, history []*pb.ChatMessage) (pb.AIService_StreamChatClient, error) {
	req := &pb.ChatRequest{
		Message: message,
		Context: contextMap,
		History: history,
	}

	return a.client.StreamChat(ctx, req)
}

// TokenHandler handles streaming tokens
type TokenHandler func(tokenType string, content string) error

// HandleUIStream processes UI generation stream
func HandleUIStream(stream pb.AIService_StreamUIClient, handler TokenHandler) error {
	for {
		token, err := stream.Recv()
		if err == io.EOF {
			break
		}
		if err != nil {
			return fmt.Errorf("stream error: %w", err)
		}

		typeStr := token.Type.String()
		if err := handler(typeStr, token.Content); err != nil {
			return err
		}
	}
	return nil
}

// HandleChatStream processes chat stream
func HandleChatStream(stream pb.AIService_StreamChatClient, handler TokenHandler) error {
	for {
		token, err := stream.Recv()
		if err == io.EOF {
			break
		}
		if err != nil {
			return fmt.Errorf("stream error: %w", err)
		}

		typeStr := token.Type.String()
		if err := handler(typeStr, token.Content); err != nil {
			return err
		}
	}
	return nil
}
