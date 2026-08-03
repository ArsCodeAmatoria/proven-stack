package health

import (
	"encoding/json"
	"net/http"
	"sync/atomic"
)

// Checker reports dependency readiness for /readyz.
type Checker struct {
	Service  string
	NATS     atomic.Bool
	Temporal atomic.Bool
	Optional bool
}

type Status struct {
	Status  string `json:"status"`
	Service string `json:"service"`
}

type ReadyStatus struct {
	Status   string `json:"status"`
	Service  string `json:"service"`
	NATS     bool   `json:"nats"`
	Temporal bool   `json:"temporal"`
}

func WriteJSON(w http.ResponseWriter, status int, body any) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(body)
}

// Mount registers /health, /healthz, and /readyz.
func Mount(mux *http.ServeMux, c *Checker) {
	live := func(w http.ResponseWriter, _ *http.Request) {
		WriteJSON(w, http.StatusOK, Status{Status: "ok", Service: c.Service})
	}
	mux.HandleFunc("/health", live)
	mux.HandleFunc("/healthz", live)

	mux.HandleFunc("/readyz", func(w http.ResponseWriter, _ *http.Request) {
		natsOK := c.NATS.Load()
		temporalOK := c.Temporal.Load()
		ready := natsOK && temporalOK
		body := ReadyStatus{
			Status:   "ready",
			Service:  c.Service,
			NATS:     natsOK,
			Temporal: temporalOK,
		}
		if !ready {
			body.Status = "degraded"
		}
		if c.Optional || ready {
			WriteJSON(w, http.StatusOK, body)
			return
		}
		WriteJSON(w, http.StatusServiceUnavailable, body)
	})
}
