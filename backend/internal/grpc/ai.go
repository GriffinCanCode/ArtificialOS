package grpc

import (
	"context"
	"fmt"
	"io"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"

	pb "github.com/GriffinCanCode/AgentOS/backend/proto/ai"
)

// AIClient wraps gRPC client for AI service communication
type AIClient struct {
	conn   *grpc.ClientConn
	client pb.AIServiceClient
	addr   string
}

// NewAIClient creates a new AI client
func NewAIClient(addr string) (*AIClient, error) {
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	conn, err := grpc.DialContext(ctx, addr,
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithBlock(),
	)
	if err != nil {
		return nil, fmt.Errorf("failed to connect to AI service: %w", err)
	}

	return &AIClient{
		conn:   conn,
		client: pb.NewAIServiceClient(conn),
		addr:   addr,
	}, nil
}

// Close closes the connection
func (a *AIClient) Close() error {
	if a.conn != nil {
		return a.conn.Close()
	}
	return nil
}

// GenerateUI generates a UI specification
func (a *AIClient) GenerateUI(message string, contextMap map[string]string, parentID *string) (*pb.UIResponse, error) {
	ctx, cancel := context.WithTimeout(context.Background(), 60*time.Second)
	defer cancel()

	req := &pb.UIRequest{
		Message: message,
		Context: contextMap,
	}
	if parentID != nil {
		req.ParentId = parentID
	}

	return a.client.GenerateUI(ctx, req)
}

// StreamUI streams UI generation with real-time updates
func (a *AIClient) StreamUI(message string, contextMap map[string]string, parentID *string) (pb.AIService_StreamUIClient, error) {
	req := &pb.UIRequest{
		Message: message,
		Context: contextMap,
	}
	if parentID != nil {
		req.ParentId = parentID
	}

	return a.client.StreamUI(context.Background(), req)
}

// StreamChat streams chat response
func (a *AIClient) StreamChat(message string, contextMap map[string]string, history []*pb.ChatMessage) (pb.AIService_StreamChatClient, error) {
	req := &pb.ChatRequest{
		Message: message,
		Context: contextMap,
		History: history,
	}

	return a.client.StreamChat(context.Background(), req)
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
