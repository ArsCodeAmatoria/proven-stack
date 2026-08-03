package httpx

import (
	"context"
	"net/http"
	"time"

	"github.com/google/uuid"

	"github.com/ArsCodeAmatoria/proven-stack/go/internal/platform/metrics"
)

type ctxKey string

const (
	RequestIDHeader     = "X-Request-Id"
	CorrelationIDHeader = "X-Correlation-Id"

	requestIDKey     ctxKey = "request_id"
	correlationIDKey ctxKey = "correlation_id"
)

// RequestIDFromContext returns the request id if present.
func RequestIDFromContext(ctx context.Context) string {
	v, _ := ctx.Value(requestIDKey).(string)
	return v
}

// CorrelationIDFromContext returns the correlation id if present.
func CorrelationIDFromContext(ctx context.Context) string {
	v, _ := ctx.Value(correlationIDKey).(string)
	return v
}

// Middleware injects request/correlation ids and records HTTP RED metrics.
func Middleware(service string, reg *metrics.Registry) func(http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			requestID := headerOrNew(r.Header.Get(RequestIDHeader))
			correlationID := r.Header.Get(CorrelationIDHeader)
			if correlationID == "" {
				correlationID = requestID
			}

			ctx := context.WithValue(r.Context(), requestIDKey, requestID)
			ctx = context.WithValue(ctx, correlationIDKey, correlationID)
			r = r.WithContext(ctx)

			w.Header().Set(RequestIDHeader, requestID)
			w.Header().Set(CorrelationIDHeader, correlationID)

			rw := &statusRecorder{ResponseWriter: w, status: http.StatusOK}
			started := time.Now()
			next.ServeHTTP(rw, r)

			if reg != nil && reg.HTTP != nil {
				class := metrics.StatusClass(rw.status)
				reg.HTTP.Requests.WithLabelValues(service, r.Method, class).Inc()
				reg.HTTP.Duration.WithLabelValues(service, r.Method, class).Observe(time.Since(started).Seconds())
			}
		})
	}
}

func headerOrNew(raw string) string {
	if raw = trim(raw); raw != "" {
		return raw
	}
	return uuid.NewString()
}

func trim(s string) string {
	for len(s) > 0 && (s[0] == ' ' || s[0] == '\t') {
		s = s[1:]
	}
	for len(s) > 0 && (s[len(s)-1] == ' ' || s[len(s)-1] == '\t') {
		s = s[:len(s)-1]
	}
	return s
}

type statusRecorder struct {
	http.ResponseWriter
	status int
}

func (r *statusRecorder) WriteHeader(code int) {
	r.status = code
	r.ResponseWriter.WriteHeader(code)
}
