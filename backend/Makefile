.PHONY: build test run proto clean

# Build the server
build:
	go build -o bin/server ./cmd/server

# Run tests
test:
	go test -v ./...

# Run the server
run:
	go run ./cmd/server -port 8000 -kernel localhost:50051 -ai localhost:50052

# Generate protobuf code
proto:
	protoc --go_out=. --go_opt=paths=source_relative \
		--go-grpc_out=. --go-grpc_opt=paths=source_relative \
		proto/ai.proto
	cd ../proto && protoc --go_out=../go-service/proto/kernel --go_opt=paths=source_relative \
		--go-grpc_out=../go-service/proto/kernel --go-grpc_opt=paths=source_relative \
		kernel.proto

# Clean build artifacts
clean:
	rm -rf bin/
	go clean

# Install dependencies
deps:
	go mod download
	go mod tidy

# Format code
fmt:
	go fmt ./...

# Lint code
lint:
	golangci-lint run ./...

