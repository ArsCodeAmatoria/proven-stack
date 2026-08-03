package tracing

import (
	"context"
	"fmt"
	"log/slog"
	"strings"

	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracehttp"
	"go.opentelemetry.io/otel/propagation"
	"go.opentelemetry.io/otel/sdk/resource"
	sdktrace "go.opentelemetry.io/otel/sdk/trace"
)

// Config controls OpenTelemetry hooks for workers.
type Config struct {
	Enabled     bool
	Endpoint    string
	ServiceName string
	Version     string
	SampleRatio float64
}

// Setup installs a TracerProvider when enabled. Returns a shutdown func.
func Setup(ctx context.Context, cfg Config, log *slog.Logger) (func(context.Context) error, error) {
	otel.SetTextMapPropagator(propagation.NewCompositeTextMapPropagator(
		propagation.TraceContext{},
		propagation.Baggage{},
	))

	if !cfg.Enabled || strings.TrimSpace(cfg.Endpoint) == "" {
		log.Info("opentelemetry tracing disabled", "service", cfg.ServiceName)
		return func(context.Context) error { return nil }, nil
	}

	endpoint := normalizeEndpoint(cfg.Endpoint)
	exporter, err := otlptracehttp.New(ctx,
		otlptracehttp.WithEndpointURL(endpoint),
	)
	if err != nil {
		return nil, fmt.Errorf("otlp http exporter: %w", err)
	}

	res, err := resource.New(ctx,
		resource.WithAttributes(
			attribute.String("service.name", cfg.ServiceName),
			attribute.String("service.version", cfg.Version),
		),
	)
	if err != nil {
		return nil, fmt.Errorf("otel resource: %w", err)
	}

	ratio := cfg.SampleRatio
	if ratio <= 0 {
		ratio = 0
	}
	if ratio > 1 {
		ratio = 1
	}

	tp := sdktrace.NewTracerProvider(
		sdktrace.WithBatcher(exporter),
		sdktrace.WithResource(res),
		sdktrace.WithSampler(sdktrace.ParentBased(sdktrace.TraceIDRatioBased(ratio))),
	)
	otel.SetTracerProvider(tp)
	log.Info("opentelemetry tracing enabled",
		"service", cfg.ServiceName,
		"endpoint", endpoint,
		"sample_ratio", ratio,
	)

	return tp.Shutdown, nil
}

func normalizeEndpoint(raw string) string {
	trimmed := strings.TrimRight(strings.TrimSpace(raw), "/")
	if strings.HasSuffix(trimmed, "/v1/traces") {
		return trimmed
	}
	return trimmed + "/v1/traces"
}
