// Package temporalx bootstraps Temporal workers without registering workflows yet.
package temporalx

import (
	"fmt"
	"sync"

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

// WorkflowDefinition is metadata for a future workflow registration (no executable yet).
type WorkflowDefinition struct {
	Name        string
	TaskQueue   string
	Version     string
	Description string
}

// ActivityDefinition is metadata for a future activity registration (no executable yet).
type ActivityDefinition struct {
	Name        string
	TaskQueue   string
	Version     string
	Description string
}

// WorkflowRegistry tracks workflow metadata. Empty until workflows land.
type WorkflowRegistry struct {
	mu      sync.RWMutex
	entries map[string]WorkflowDefinition
}

// NewWorkflowRegistry returns an empty registry.
func NewWorkflowRegistry() *WorkflowRegistry {
	return &WorkflowRegistry{entries: make(map[string]WorkflowDefinition)}
}

// Register adds workflow metadata. Duplicate names are rejected.
func (r *WorkflowRegistry) Register(def WorkflowDefinition) error {
	if def.Name == "" {
		return fmt.Errorf("workflow name must not be empty")
	}
	r.mu.Lock()
	defer r.mu.Unlock()
	if _, ok := r.entries[def.Name]; ok {
		return fmt.Errorf("workflow %q is already registered", def.Name)
	}
	r.entries[def.Name] = def
	return nil
}

// Len returns the number of registered workflow definitions.
func (r *WorkflowRegistry) Len() int {
	r.mu.RLock()
	defer r.mu.RUnlock()
	return len(r.entries)
}

// Names returns registered workflow names.
func (r *WorkflowRegistry) Names() []string {
	r.mu.RLock()
	defer r.mu.RUnlock()
	out := make([]string, 0, len(r.entries))
	for name := range r.entries {
		out = append(out, name)
	}
	return out
}

// ActivityRegistry tracks activity metadata. Empty until activities land.
type ActivityRegistry struct {
	mu      sync.RWMutex
	entries map[string]ActivityDefinition
}

// NewActivityRegistry returns an empty registry.
func NewActivityRegistry() *ActivityRegistry {
	return &ActivityRegistry{entries: make(map[string]ActivityDefinition)}
}

// Register adds activity metadata. Duplicate names are rejected.
func (r *ActivityRegistry) Register(def ActivityDefinition) error {
	if def.Name == "" {
		return fmt.Errorf("activity name must not be empty")
	}
	r.mu.Lock()
	defer r.mu.Unlock()
	if _, ok := r.entries[def.Name]; ok {
		return fmt.Errorf("activity %q is already registered", def.Name)
	}
	r.entries[def.Name] = def
	return nil
}

// Len returns the number of registered activity definitions.
func (r *ActivityRegistry) Len() int {
	r.mu.RLock()
	defer r.mu.RUnlock()
	return len(r.entries)
}

// Names returns registered activity names.
func (r *ActivityRegistry) Names() []string {
	r.mu.RLock()
	defer r.mu.RUnlock()
	out := make([]string, 0, len(r.entries))
	for name := range r.entries {
		out = append(out, name)
	}
	return out
}

// Handle owns the Temporal client, worker, and empty registries (foundation).
type Handle struct {
	Client     client.Client
	Worker     worker.Worker
	Opts       Options
	Workflows  *WorkflowRegistry
	Activities *ActivityRegistry
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
		// Foundation: no workflows/activities registered on the SDK worker yet.
	})

	return &Handle{
		Client:     c,
		Worker:     w,
		Opts:       opts,
		Workflows:  NewWorkflowRegistry(),
		Activities: NewActivityRegistry(),
	}, nil
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

// HealthDetail summarizes infrastructure status (empty registries are expected).
func (h *Handle) HealthDetail() string {
	if !h.Ready() {
		return "temporal handle not ready"
	}
	wf := 0
	act := 0
	if h.Workflows != nil {
		wf = h.Workflows.Len()
	}
	if h.Activities != nil {
		act = h.Activities.Len()
	}
	if wf == 0 && act == 0 {
		return fmt.Sprintf(
			"reachable; infrastructure only (task_queue=%s; no workflows/activities registered)",
			h.Opts.TaskQueue,
		)
	}
	return fmt.Sprintf(
		"reachable; workflows=%d activities=%d task_queue=%s",
		wf, act, h.Opts.TaskQueue,
	)
}
