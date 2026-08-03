package config

import (
	"fmt"
	"os"
)

type Server struct {
	Host    string
	Port    string
	Service string
}

// servicePortEnv maps binary names to preferred env vars (see .env.example).
var servicePortEnv = map[string]string{
	"notify-worker":      "PROVEN_WORKER_NOTIFY_PORT",
	"temporal-io-worker": "PROVEN_WORKER_TEMPORAL_IO_PORT",
	"media-worker":       "PROVEN_WORKER_MEDIA_PORT",
	"analytics-worker":   "PROVEN_WORKER_ANALYTICS_PORT",
}

func Load(service, defaultPort string) Server {
	host := getenv("PROVEN_WORKER_HOST", "0.0.0.0")
	specific := ""
	if key, ok := servicePortEnv[service]; ok {
		specific = os.Getenv(key)
	}
	port := firstNonEmpty(specific, os.Getenv("PROVEN_WORKER_PORT"), defaultPort)
	return Server{Host: host, Port: port, Service: service}
}

func (s Server) Addr() string {
	return fmt.Sprintf("%s:%s", s.Host, s.Port)
}

func getenv(key, fallback string) string {
	if v := os.Getenv(key); v != "" {
		return v
	}
	return fallback
}

func firstNonEmpty(values ...string) string {
	for _, v := range values {
		if v != "" {
			return v
		}
	}
	return ""
}
