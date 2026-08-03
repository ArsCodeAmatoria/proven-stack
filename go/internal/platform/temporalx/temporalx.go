// Package temporalx bootstraps Temporal workers without registering workflows yet.
package temporalx

import (
	"fmt"

	"go.temporal.io/sdk/client"
	"go.temporal.io/sdk/worker"
)

// Options configures Temporal client + worker bootstrap.
type Options struct {
	Address   string
	Namespace string
	TaskQueue string
	Identity  string
}

// Handle owns the Temporal client and worker (no workflows registered in foundation).
type Handle struct {
	Client client.Client
	Worker worker.Worker
	Opts   Options
}

// Connect dials Temporal and constructs an empty worker bound to the task queue.
func Connect(opts Options) (*Handle, error) {
	if opts.Address == "" {
		return nil, fmt.Errorf("temporal address is required")
	}
	if opts.Namespace == "" {
		opts.Namespace = "default"
	}
	if opts.TaskQueue == "" {
		return nil, fmt.Errorf("temporal task queue is required")
	}

	c, err := client.Dial(client.Options{
		HostPort:  opts.Address,
		Namespace: opts.Namespace,
		Identity:  opts.Identity,
	})
	if err != nil {
		return nil, fmt.Errorf("temporal dial: %w", err)
	}

	w := worker.New(c, opts.TaskQueue, worker.Options{
		Identity: opts.Identity,
		// Foundation: no workflows/activities registered yet.
	})

	return &Handle{Client: c, Worker: w, Opts: opts}, nil
}

// Start begins polling the task queue. Safe with zero registrations.
func (h *Handle) Start() error {
	if h == nil || h.Worker == nil {
		return fmt.Errorf("temporal worker is nil")
	}
	return h.Worker.Start()
}

// Stop stops the worker and closes the client.
func (h *Handle) Stop() {
	if h == nil {
		return
	}
	if h.Worker != nil {
		h.Worker.Stop()
	}
	if h.Client != nil {
		h.Client.Close()
	}
}

// Ready is true when a client was established.
func (h *Handle) Ready() bool {
	return h != nil && h.Client != nil && h.Worker != nil
}
