// Package app wires worker binaries: config, clients, health, registration, shutdown.
// No workflows or business domain logic.
package app

import (
	"context"
	"errors"
	"fmt"
	"log/slog"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/nats-io/nats.go"

	"github.com/ArsCodeAmatoria/proven-stack/go/internal/config"
	"github.com/ArsCodeAmatoria/proven-stack/go/internal/platform/health"
	"github.com/ArsCodeAmatoria/proven-stack/go/internal/platform/httpx"
	"github.com/ArsCodeAmatoria/proven-stack/go/internal/platform/logging"
	"github.com/ArsCodeAmatoria/proven-stack/go/internal/platform/metrics"
	"github.com/ArsCodeAmatoria/proven-stack/go/internal/platform/natsx"
	"github.com/ArsCodeAmatoria/proven-stack/go/internal/platform/retry"
	"github.com/ArsCodeAmatoria/proven-stack/go/internal/platform/temporalx"
	"github.com/ArsCodeAmatoria/proven-stack/go/internal/platform/tracing"
)

// Run boots a foundation worker process until SIGINT/SIGTERM.
func Run(service, defaultPort string) {
	cfg := config.MustLoad(service, defaultPort)
	log := logging.New(cfg)
	slog.SetDefault(log)

	log.Info("configuration loaded", "config", cfg.Redacted())

	ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer stop()

	otelShutdown, err := tracing.Setup(ctx, tracing.Config{
		Enabled:     cfg.Observability.OTelEnabled,
		Endpoint:    cfg.Observability.OTelEndpoint,
		ServiceName: cfg.Service,
		Version:     cfg.Observability.ServiceVersion,
		SampleRatio: cfg.Observability.OTelSampleRatio,
	}, log)
	if err != nil {
		log.Error("opentelemetry setup failed", "error", err)
		os.Exit(1)
	}
	defer func() {
		shutdownCtx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()
		if err := otelShutdown(shutdownCtx); err != nil {
			log.Warn("opentelemetry shutdown", "error", err)
		}
	}()

	var reg *metrics.Registry
	if cfg.Observability.MetricsEnabled {
		reg = metrics.New(cfg.Service)
	}

	checker := &health.Checker{Service: cfg.Service, Optional: cfg.InfraOptional}

	nc, temporal, err := connectInfra(ctx, cfg, log, checker)
	if err != nil {
		log.Error("infrastructure bootstrap failed", "error", err)
		os.Exit(1)
	}
	defer func() {
		if temporal != nil {
			temporal.Stop()
		}
		if nc != nil {
			_ = nc.Drain()
			nc.Close()
		}
	}()

	if temporal != nil {
		if err := temporal.Start(); err != nil {
			log.Error("temporal worker start failed", "error", err)
			if !cfg.InfraOptional {
				os.Exit(1)
			}
			checker.Temporal.Store(false)
			log.Warn("continuing without temporal worker (infra optional)")
		} else {
			log.Info("temporal worker registered",
				"task_queue", cfg.Temporal.TaskQueue,
				"namespace", cfg.Temporal.Namespace,
				"workflows", 0,
				"activities", 0,
			)
		}
	}

	mux := http.NewServeMux()
	health.Mount(mux, checker)
	if reg != nil {
		mux.Handle("/metrics", reg.Handler())
	}
	mux.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/" {
			http.NotFound(w, r)
			return
		}
		health.WriteJSON(w, http.StatusOK, map[string]any{
			"service":     cfg.Service,
			"status":      "running",
			"mode":        "foundation",
			"environment": string(cfg.Environment),
			"task_queue":  cfg.Temporal.TaskQueue,
			"nats":        checker.NATS.Load(),
			"temporal":    checker.Temporal.Load(),
			"metrics":     cfg.Observability.MetricsEnabled,
			"otel":        cfg.Observability.OTelEnabled,
			"retry": map[string]any{
				"max_attempts": cfg.Retry.MaxAttempts,
				"initial_ms":   cfg.Retry.InitialMS,
				"max_ms":       cfg.Retry.MaxMS,
			},
		})
	})

	handler := httpx.Middleware(cfg.Service, reg)(mux)

	server := &http.Server{
		Addr:              cfg.Server.Addr(),
		Handler:           handler,
		ReadHeaderTimeout: 5 * time.Second,
	}

	errCh := make(chan error, 1)
	go func() {
		log.Info("health server listening", "addr", cfg.Server.Addr())
		if err := server.ListenAndServe(); err != nil && !errors.Is(err, http.ErrServerClosed) {
			errCh <- err
		}
		close(errCh)
	}()

	select {
	case <-ctx.Done():
		log.Info("shutdown signal received")
	case err := <-errCh:
		if err != nil {
			log.Error("health server failed", "error", err)
			os.Exit(1)
		}
	}

	shutdownCtx, cancel := context.WithTimeout(context.Background(), cfg.Shutdown)
	defer cancel()
	if err := server.Shutdown(shutdownCtx); err != nil {
		log.Warn("health server shutdown", "error", err)
	}
	log.Info("worker stopped cleanly")
}

func connectInfra(
	ctx context.Context,
	cfg config.Config,
	log *slog.Logger,
	checker *health.Checker,
) (*nats.Conn, *temporalx.Handle, error) {
	policy := retry.Policy{
		MaxAttempts: cfg.Retry.MaxAttempts,
		Initial:     time.Duration(cfg.Retry.InitialMS) * time.Millisecond,
		Max:         time.Duration(cfg.Retry.MaxMS) * time.Millisecond,
		Multiplier:  2,
		Jitter:      0.2,
	}
	if cfg.InfraOptional && policy.MaxAttempts > 2 {
		policy.MaxAttempts = 2
	}

	var nc *nats.Conn
	err := retry.Do(ctx, policy, func(context.Context) error {
		conn, err := natsx.Connect(natsx.Options{
			URL:  cfg.NatsURL,
			Name: cfg.Service,
		})
		if err != nil {
			return err
		}
		nc = conn
		return nil
	})
	if err != nil {
		if cfg.InfraOptional {
			log.Warn("nats unavailable (infra optional)", "error", err)
		} else {
			return nil, nil, fmt.Errorf("nats: %w", err)
		}
	} else {
		checker.NATS.Store(true)
		log.Info("nats connected", "url", cfg.NatsURL)
	}

	var temporal *temporalx.Handle
	err = retry.Do(ctx, policy, func(context.Context) error {
		h, err := temporalx.Connect(temporalx.Options{
			Address:   cfg.Temporal.Address,
			Namespace: cfg.Temporal.Namespace,
			TaskQueue: cfg.Temporal.TaskQueue,
			Identity:  cfg.Service,
		})
		if err != nil {
			return err
		}
		temporal = h
		return nil
	})
	if err != nil {
		if cfg.InfraOptional {
			log.Warn("temporal unavailable (infra optional)", "error", err)
		} else {
			if nc != nil {
				nc.Close()
			}
			return nil, nil, fmt.Errorf("temporal: %w", err)
		}
	} else {
		checker.Temporal.Store(true)
		log.Info("temporal client connected",
			"address", cfg.Temporal.Address,
			"namespace", cfg.Temporal.Namespace,
		)
	}

	return nc, temporal, nil
}
