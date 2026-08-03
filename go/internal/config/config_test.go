package config_test

import (
	"os"
	"testing"

	"github.com/ArsCodeAmatoria/proven-stack/go/internal/config"
)

func TestDevelopmentDefaults(t *testing.T) {
	t.Setenv("PROVEN_ENV", "development")
	os.Unsetenv("DATABASE_URL")
	os.Unsetenv("PROVEN_SESSION_SECRET")

	cfg, err := config.Load("notify-worker", "8091")
	if err != nil {
		t.Fatalf("Load: %v", err)
	}
	if cfg.Environment != config.Development {
		t.Fatalf("env=%s", cfg.Environment)
	}
	if cfg.Server.Port != "8091" {
		t.Fatalf("port=%s", cfg.Server.Port)
	}
	if got := cfg.Redacted(); !contains(got, "[REDACTED]") {
		t.Fatalf("expected redacted summary, got %s", got)
	}
}

func TestProductionMissing(t *testing.T) {
	t.Setenv("PROVEN_ENV", "production")
	os.Unsetenv("DATABASE_URL")
	os.Unsetenv("REDIS_URL")
	os.Unsetenv("NATS_URL")
	os.Unsetenv("TEMPORAL_ADDRESS")
	os.Unsetenv("PROVEN_SESSION_SECRET")

	_, err := config.Load("notify-worker", "8091")
	if err == nil {
		t.Fatal("expected missing config error")
	}
}

func TestProductionRejectsWeakSecrets(t *testing.T) {
	t.Setenv("PROVEN_ENV", "production")
	t.Setenv("DATABASE_URL", "postgres://proven:proven@db.example.com:5432/proven")
	t.Setenv("REDIS_URL", "redis://redis.example.com:6379")
	t.Setenv("NATS_URL", "nats://nats.example.com:4222")
	t.Setenv("TEMPORAL_ADDRESS", "temporal.example.com:7233")
	t.Setenv("PROVEN_SESSION_SECRET", "short")

	_, err := config.Load("notify-worker", "8091")
	if err == nil {
		t.Fatal("expected secrets validation error")
	}
}

func TestProductionAcceptsStrongConfig(t *testing.T) {
	t.Setenv("PROVEN_ENV", "production")
	t.Setenv("DATABASE_URL", "postgres://app:s3cure-P@ssw0rd-long@db.internal:5432/proven")
	t.Setenv("REDIS_URL", "rediss://:token@redis.internal:6379")
	t.Setenv("NATS_URL", "tls://nats.internal:4222")
	t.Setenv("TEMPORAL_ADDRESS", "temporal.internal:7233")
	t.Setenv("PROVEN_SESSION_SECRET", "production-session-secret-value-32chars-min")

	cfg, err := config.Load("notify-worker", "8091")
	if err != nil {
		t.Fatalf("Load: %v", err)
	}
	if !cfg.Environment.IsProduction() {
		t.Fatal("expected production")
	}
}

func contains(s, sub string) bool {
	return len(s) >= len(sub) && (s == sub || len(sub) == 0 ||
		(func() bool {
			for i := 0; i+len(sub) <= len(s); i++ {
				if s[i:i+len(sub)] == sub {
					return true
				}
			}
			return false
		})())
}
