package logging

import (
	"log/slog"
	"os"
	"strings"

	"github.com/ArsCodeAmatoria/proven-stack/go/internal/config"
)

// New builds a structured slog logger for the worker process.
func New(cfg config.Config) *slog.Logger {
	level := parseLevel(cfg.LogLevel)
	opts := &slog.HandlerOptions{Level: level}

	var handler slog.Handler
	if cfg.Environment.IsProduction() {
		handler = slog.NewJSONHandler(os.Stdout, opts)
	} else {
		handler = slog.NewTextHandler(os.Stdout, opts)
	}

	return slog.New(handler).With(
		"service", cfg.Service,
		"version", cfg.Observability.ServiceVersion,
		"env", string(cfg.Environment),
	)
}

func parseLevel(raw string) slog.Level {
	switch strings.ToLower(strings.TrimSpace(raw)) {
	case "debug":
		return slog.LevelDebug
	case "warn", "warning":
		return slog.LevelWarn
	case "error":
		return slog.LevelError
	default:
		return slog.LevelInfo
	}
}
