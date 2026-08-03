package metrics

import (
	"net/http"

	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/collectors"
	"github.com/prometheus/client_golang/prometheus/promhttp"
)

// Registry is the process-wide Prometheus registry for foundation metrics.
type Registry struct {
	registry *prometheus.Registry
	HTTP     *HTTPMetrics
}

// HTTPMetrics holds low-cardinality HTTP RED metrics.
type HTTPMetrics struct {
	Requests *prometheus.CounterVec
	Duration *prometheus.HistogramVec
}

// New builds a registry with process collectors and HTTP RED metrics.
func New(service string) *Registry {
	reg := prometheus.NewRegistry()
	reg.MustRegister(
		collectors.NewGoCollector(),
		collectors.NewProcessCollector(collectors.ProcessCollectorOpts{}),
	)

	httpMetrics := &HTTPMetrics{
		Requests: prometheus.NewCounterVec(
			prometheus.CounterOpts{
				Name: "http_server_requests_total",
				Help: "Total HTTP requests handled by the worker health server.",
			},
			[]string{"service", "method", "status_class"},
		),
		Duration: prometheus.NewHistogramVec(
			prometheus.HistogramOpts{
				Name:    "http_server_request_duration_seconds",
				Help:    "HTTP request duration in seconds.",
				Buckets: prometheus.DefBuckets,
			},
			[]string{"service", "method", "status_class"},
		),
	}
	reg.MustRegister(httpMetrics.Requests, httpMetrics.Duration)

	// Warm zero series for the service label.
	_ = service

	return &Registry{registry: reg, HTTP: httpMetrics}
}

// Handler returns the Prometheus scrape handler.
func (r *Registry) Handler() http.Handler {
	return promhttp.HandlerFor(r.registry, promhttp.HandlerOpts{})
}

// StatusClass maps an HTTP status code to a low-cardinality class.
func StatusClass(code int) string {
	switch {
	case code >= 100 && code < 200:
		return "1xx"
	case code >= 200 && code < 300:
		return "2xx"
	case code >= 300 && code < 400:
		return "3xx"
	case code >= 400 && code < 500:
		return "4xx"
	default:
		return "5xx"
	}
}
