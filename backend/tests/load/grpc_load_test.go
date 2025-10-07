//go:build load
// +build load

package main

import (
	"context"
	"flag"
	"fmt"
	"log"
	"sync"
	"sync/atomic"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"

	kernelPb "github.com/GriffinCanCode/AgentOS/backend/proto/kernel"
)

var (
	addr     = flag.String("addr", "localhost:50051", "gRPC server address")
	requests = flag.Int("requests", 1000, "Total number of requests")
	workers  = flag.Int("workers", 10, "Number of concurrent workers")
)

type result struct {
	duration time.Duration
	err      error
}

func main() {
	flag.Parse()

	log.Printf("Starting gRPC load test")
	log.Printf("Target: %s", *addr)
	log.Printf("Requests: %d", *requests)
	log.Printf("Workers: %d", *workers)

	// Connect to server
	conn, err := grpc.Dial(
		*addr,
		grpc.WithTransportCredentials(insecure.NewCredentials()),
	)
	if err != nil {
		log.Fatalf("Failed to connect: %v", err)
	}
	defer conn.Close()

	client := kernelPb.NewKernelServiceClient(conn)

	// Run load test
	results := runLoadTest(client, *requests, *workers)

	// Analyze results
	analyzeResults(results)
}

func runLoadTest(client kernelPb.KernelServiceClient, totalRequests, workers int) []result {
	results := make([]result, 0, totalRequests)
	var mu sync.Mutex

	var completed atomic.Int32
	start := time.Now()

	var wg sync.WaitGroup
	requestsChan := make(chan int, totalRequests)

	// Populate requests channel
	for i := 0; i < totalRequests; i++ {
		requestsChan <- i
	}
	close(requestsChan)

	// Start workers
	for w := 0; w < workers; w++ {
		wg.Add(1)
		go func(workerID int) {
			defer wg.Done()

			for range requestsChan {
				res := executeRequest(client)

				mu.Lock()
				results = append(results, res)
				mu.Unlock()

				count := completed.Add(1)
				if count%100 == 0 {
					elapsed := time.Since(start)
					rps := float64(count) / elapsed.Seconds()
					log.Printf("Progress: %d/%d requests (%.2f req/sec)",
						count, totalRequests, rps)
				}
			}
		}(w)
	}

	wg.Wait()

	return results
}

func executeRequest(client kernelPb.KernelServiceClient) result {
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	start := time.Now()

	req := &kernelPb.CreateProcessRequest{
		Command: "load_test",
	}

	_, err := client.CreateProcess(ctx, req)

	return result{
		duration: time.Since(start),
		err:      err,
	}
}

func analyzeResults(results []result) {
	if len(results) == 0 {
		log.Println("No results to analyze")
		return
	}

	var (
		totalDuration time.Duration
		successCount  int
		errorCount    int
		durations     []time.Duration
	)

	for _, r := range results {
		totalDuration += r.duration
		if r.err == nil {
			successCount++
		} else {
			errorCount++
		}
		durations = append(durations, r.duration)
	}

	// Sort durations for percentile calculation
	sortDurations(durations)

	total := len(results)
	avgDuration := totalDuration / time.Duration(total)
	p50 := durations[total*50/100]
	p95 := durations[total*95/100]
	p99 := durations[total*99/100]
	maxDuration := durations[total-1]

	fmt.Println("\n========================================")
	fmt.Println("Load Test Results")
	fmt.Println("========================================")
	fmt.Printf("Total Requests:    %d\n", total)
	fmt.Printf("Successful:        %d (%.2f%%)\n", successCount, float64(successCount)/float64(total)*100)
	fmt.Printf("Failed:            %d (%.2f%%)\n", errorCount, float64(errorCount)/float64(total)*100)
	fmt.Println("----------------------------------------")
	fmt.Printf("Average Latency:   %v\n", avgDuration)
	fmt.Printf("P50 Latency:       %v\n", p50)
	fmt.Printf("P95 Latency:       %v\n", p95)
	fmt.Printf("P99 Latency:       %v\n", p99)
	fmt.Printf("Max Latency:       %v\n", maxDuration)
	fmt.Println("========================================")
}

func sortDurations(durations []time.Duration) {
	// Simple bubble sort (good enough for test purposes)
	n := len(durations)
	for i := 0; i < n-1; i++ {
		for j := 0; j < n-i-1; j++ {
			if durations[j] > durations[j+1] {
				durations[j], durations[j+1] = durations[j+1], durations[j]
			}
		}
	}
}
