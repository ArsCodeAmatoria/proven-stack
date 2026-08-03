// Package config provides typed environment configuration for Go workers.
// Development / testing / production with secrets + startup validation.
// No business domain logic.
package config

import (
	"errors"
	"fmt"
	"os"
	"strconv"
	"strings"
	"time"
)

// Environment controls defaults and secret strictness.
type Environment string

const (
	Development Environment = "development"
	Testing     Environment = "testing"
	Production  Environment = "production"
)

func ParseEnvironment(raw string) (Environment, error) {
	switch strings.ToLower(strings.TrimSpace(raw)) {
	case "", "development", "dev", "local":
		return Development, nil
	case "testing", "test":
		return Testing, nil
	case "production", "prod":
		return Production, nil
	default:
		return "", fmt.Errorf("invalid PROVEN_ENV %q (expected development|testing|production)", raw)
	}
}

func (e Environment) IsProduction() bool { return e == Production }

// Config is the typed worker process configuration.
type Config struct {
	Environment   Environment
	Service       string
	Server        ServerConfig
	DatabaseURL   string // secret — never log
	RedisURL      string // secret — never log
	NatsURL       string
	Temporal      TemporalConfig
	SessionSecret string // secret — never log
	LogLevel      string
	InfraOptional bool
	Retry         RetryConfig
	Shutdown      time.Duration
	Observability ObservabilityConfig
}

// ObservabilityConfig controls logs/metrics/tracing hooks for workers.
type ObservabilityConfig struct {
	ServiceVersion  string
	MetricsEnabled  bool
	OTelEnabled     bool
	OTelEndpoint    string
	OTelSampleRatio float64
}

type ServerConfig struct {
	Host string
	Port string
}

func (s ServerConfig) Addr() string {
	return fmt.Sprintf("%s:%s", s.Host, s.Port)
}

type TemporalConfig struct {
	Address   string
	Namespace string
	TaskQueue string
}

type RetryConfig struct {
	MaxAttempts int
	InitialMS   int
	MaxMS       int
}

var servicePortEnv = map[string]string{
	"notify-worker":      "PROVEN_WORKER_NOTIFY_PORT",
	"temporal-io-worker": "PROVEN_WORKER_TEMPORAL_IO_PORT",
	"media-worker":       "PROVEN_WORKER_MEDIA_PORT",
	"analytics-worker":   "PROVEN_WORKER_ANALYTICS_PORT",
}

var defaultTaskQueues = map[string]string{
	"notify-worker":      "proven-notify",
	"temporal-io-worker": "proven-io",
	"media-worker":       "proven-media",
	"analytics-worker":   "proven-analytics",
}

const defaultDevSessionSecret = "dev-only-session-secret-change-me-32b"

// Load reads, types, and validates configuration for a worker binary.
func Load(service, defaultPort string) (Config, error) {
	env, err := ParseEnvironment(os.Getenv("PROVEN_ENV"))
	if err != nil {
		return Config{}, err
	}

	var missing []string

	host := getenv("PROVEN_WORKER_HOST", "0.0.0.0")
	port := workerPort(service, defaultPort)
	if port == "" {
		missing = append(missing, "PROVEN_WORKER_PORT")
	}

	dbURL := requireOrDefault("DATABASE_URL", env, "postgres://proven:proven@127.0.0.1:5432/proven", &missing)
	redisURL := requireOrDefault("REDIS_URL", env, "redis://127.0.0.1:6379", &missing)
	natsURL := requireOrDefault("NATS_URL", env, "nats://127.0.0.1:4222", &missing)
	temporalAddr := requireOrDefault("TEMPORAL_ADDRESS", env, "127.0.0.1:7233", &missing)
	temporalNS := getenv("TEMPORAL_NAMESPACE", "default")
	taskQueue := firstNonEmpty(
		os.Getenv("TEMPORAL_TASK_QUEUE"),
		defaultTaskQueues[service],
		"proven-"+service,
	)

	sessionSecret := os.Getenv("PROVEN_SESSION_SECRET")
	if sessionSecret == "" {
		if env == Development {
			sessionSecret = defaultDevSessionSecret
		} else {
			missing = append(missing, "PROVEN_SESSION_SECRET")
		}
	}

	logLevel := getenv("PROVEN_LOG_LEVEL", defaultLogLevel(env))
	infraOptional := parseBool("PROVEN_INFRA_OPTIONAL", env == Development)
	shutdownSec := parseInt("PROVEN_SHUTDOWN_TIMEOUT_SEC", 15)
	retryAttempts := parseInt("PROVEN_RETRY_MAX_ATTEMPTS", 5)
	retryInitial := parseInt("PROVEN_RETRY_INITIAL_MS", 200)
	retryMax := parseInt("PROVEN_RETRY_MAX_MS", 10000)

	serviceVersion := firstNonEmpty(os.Getenv("PROVEN_SERVICE_VERSION"), os.Getenv("GIT_SHA"), "0.1.0")
	otelEndpoint := firstNonEmpty(os.Getenv("OTEL_EXPORTER_OTLP_ENDPOINT"), os.Getenv("PROVEN_OTEL_ENDPOINT"))
	otelEnabled := parseBool("PROVEN_OTEL_ENABLED", otelEndpoint != "")
	otelSample := parseFloat("PROVEN_OTEL_SAMPLE_RATIO", 1.0)
	metricsEnabled := parseBool("PROVEN_METRICS_ENABLED", true)

	if len(missing) > 0 {
		return Config{}, missingError(missing)
	}

	cfg := Config{
		Environment: env,
		Service:     service,
		Server: ServerConfig{
			Host: host,
			Port: port,
		},
		DatabaseURL:   dbURL,
		RedisURL:      redisURL,
		NatsURL:       natsURL,
		Temporal:      TemporalConfig{Address: temporalAddr, Namespace: temporalNS, TaskQueue: taskQueue},
		SessionSecret: sessionSecret,
		LogLevel:      logLevel,
		InfraOptional: infraOptional,
		Retry: RetryConfig{
			MaxAttempts: retryAttempts,
			InitialMS:   retryInitial,
			MaxMS:       retryMax,
		},
		Shutdown: time.Duration(shutdownSec) * time.Second,
		Observability: ObservabilityConfig{
			ServiceVersion:  serviceVersion,
			MetricsEnabled:  metricsEnabled,
			OTelEnabled:     otelEnabled,
			OTelEndpoint:    otelEndpoint,
			OTelSampleRatio: otelSample,
		},
	}

	if err := ValidateSecrets(cfg); err != nil {
		return Config{}, err
	}
	if err := ValidateStartup(cfg); err != nil {
		return Config{}, err
	}
	return cfg, nil
}

// MustLoad is Load that terminates the process on failure (startup fail-fast).
func MustLoad(service, defaultPort string) Config {
	cfg, err := Load(service, defaultPort)
	if err != nil {
		fmt.Fprintf(os.Stderr, "configuration error: %v\n", err)
		os.Exit(1)
	}
	return cfg
}

func ValidateSecrets(cfg Config) error {
	var reasons []string
	if strings.TrimSpace(cfg.SessionSecret) == "" {
		reasons = append(reasons, "PROVEN_SESSION_SECRET is empty")
	}
	switch cfg.Environment {
	case Testing:
		if len(cfg.SessionSecret) < 16 {
			reasons = append(reasons, "PROVEN_SESSION_SECRET must be at least 16 characters in testing")
		}
		rejectPlaceholderDB(cfg.DatabaseURL, &reasons)
	case Production:
		if len(cfg.SessionSecret) < 32 {
			reasons = append(reasons, "PROVEN_SESSION_SECRET must be at least 32 characters in production")
		}
		if isWeakSessionSecret(cfg.SessionSecret) {
			reasons = append(reasons, "PROVEN_SESSION_SECRET looks like a development placeholder")
		}
		rejectPlaceholderDB(cfg.DatabaseURL, &reasons)
		rejectLoopback("DATABASE_URL", cfg.DatabaseURL, &reasons)
		rejectLoopback("REDIS_URL", cfg.RedisURL, &reasons)
		rejectLoopback("NATS_URL", cfg.NatsURL, &reasons)
		rejectLoopback("TEMPORAL_ADDRESS", cfg.Temporal.Address, &reasons)
	}
	if len(reasons) > 0 {
		return fmt.Errorf("secrets validation failed: %s", strings.Join(reasons, "; "))
	}
	return nil
}

func ValidateStartup(cfg Config) error {
	var reasons []string
	if strings.TrimSpace(cfg.Server.Host) == "" {
		reasons = append(reasons, "PROVEN_WORKER_HOST is empty")
	}
	if strings.TrimSpace(cfg.Server.Port) == "" {
		reasons = append(reasons, "worker port is empty")
	}
	if _, err := strconv.Atoi(cfg.Server.Port); err != nil {
		reasons = append(reasons, "worker port must be an integer")
	}
	if strings.TrimSpace(cfg.NatsURL) == "" {
		reasons = append(reasons, "NATS_URL is empty")
	}
	if strings.TrimSpace(cfg.Temporal.Address) == "" {
		reasons = append(reasons, "TEMPORAL_ADDRESS is empty")
	}
	if strings.TrimSpace(cfg.Temporal.Namespace) == "" {
		reasons = append(reasons, "TEMPORAL_NAMESPACE is empty")
	}
	if strings.TrimSpace(cfg.Temporal.TaskQueue) == "" {
		reasons = append(reasons, "TEMPORAL_TASK_QUEUE is empty")
	}
	if strings.TrimSpace(cfg.Service) == "" {
		reasons = append(reasons, "service name is empty")
	}
	if cfg.Retry.MaxAttempts < 1 {
		reasons = append(reasons, "PROVEN_RETRY_MAX_ATTEMPTS must be >= 1")
	}
	if cfg.Shutdown <= 0 {
		reasons = append(reasons, "PROVEN_SHUTDOWN_TIMEOUT_SEC must be > 0")
	}
	if len(reasons) > 0 {
		return fmt.Errorf("startup validation failed: %s", strings.Join(reasons, "; "))
	}
	return nil
}

// Redacted returns a log-safe summary (no secret values).
func (c Config) Redacted() string {
	return fmt.Sprintf(
		"env=%s service=%s addr=%s nats=%s temporal=%s namespace=%s queue=%s infra_optional=%t log=%s database=[REDACTED] redis=[REDACTED] session=[REDACTED]",
		c.Environment, c.Service, c.Server.Addr(), c.NatsURL, c.Temporal.Address, c.Temporal.Namespace, c.Temporal.TaskQueue, c.InfraOptional, c.LogLevel,
	)
}

func workerPort(service, defaultPort string) string {
	if key, ok := servicePortEnv[service]; ok {
		if v := os.Getenv(key); v != "" {
			return v
		}
	}
	if v := os.Getenv("PROVEN_WORKER_PORT"); v != "" {
		return v
	}
	return defaultPort
}

func requireOrDefault(key string, env Environment, devDefault string, missing *[]string) string {
	if v := os.Getenv(key); v != "" {
		return v
	}
	if env == Development {
		return devDefault
	}
	*missing = append(*missing, key)
	return ""
}

func getenv(key, fallback string) string {
	if v := os.Getenv(key); v != "" {
		return v
	}
	return fallback
}

func firstNonEmpty(values ...string) string {
	for _, v := range values {
		if strings.TrimSpace(v) != "" {
			return v
		}
	}
	return ""
}

func defaultLogLevel(env Environment) string {
	if env == Development {
		return "debug"
	}
	return "info"
}

func parseBool(key string, defaultValue bool) bool {
	v := strings.ToLower(strings.TrimSpace(os.Getenv(key)))
	if v == "" {
		return defaultValue
	}
	switch v {
	case "1", "true", "yes", "on":
		return true
	case "0", "false", "no", "off":
		return false
	default:
		return defaultValue
	}
}

func parseInt(key string, defaultValue int) int {
	v := strings.TrimSpace(os.Getenv(key))
	if v == "" {
		return defaultValue
	}
	n, err := strconv.Atoi(v)
	if err != nil {
		return defaultValue
	}
	return n
}

func parseFloat(key string, defaultValue float64) float64 {
	v := strings.TrimSpace(os.Getenv(key))
	if v == "" {
		return defaultValue
	}
	n, err := strconv.ParseFloat(v, 64)
	if err != nil {
		return defaultValue
	}
	if n < 0 {
		return 0
	}
	if n > 1 {
		return 1
	}
	return n
}

// ErrMissing is a sentinel for missing required configuration keys.
var ErrMissing = errors.New("missing required configuration")

func missingError(keys []string) error {
	return fmt.Errorf("%w: %s", ErrMissing, strings.Join(keys, ", "))
}

func rejectPlaceholderDB(url string, reasons *[]string) {
	lower := strings.ToLower(url)
	if strings.Contains(lower, "proven:proven@") ||
		strings.Contains(lower, ":changeme@") ||
		strings.Contains(lower, ":password@") ||
		strings.Contains(lower, ":secret@") {
		*reasons = append(*reasons, "DATABASE_URL must not use placeholder credentials outside development")
	}
}

func rejectLoopback(key, value string, reasons *[]string) {
	lower := strings.ToLower(value)
	if strings.Contains(lower, "127.0.0.1") ||
		strings.Contains(lower, "localhost") ||
		strings.Contains(lower, "[::1]") ||
		strings.Contains(lower, "0.0.0.0") {
		*reasons = append(*reasons, fmt.Sprintf("%s must not point at localhost/loopback in production", key))
	}
}

func isWeakSessionSecret(value string) bool {
	lower := strings.ToLower(value)
	return strings.Contains(lower, "dev-only") ||
		strings.Contains(lower, "change-me") ||
		strings.Contains(lower, "changeme") ||
		lower == "secret" ||
		lower == "password"
}
