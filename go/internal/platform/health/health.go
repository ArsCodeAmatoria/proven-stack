package health

import (
	"encoding/json"
	"net/http"
)

type Status struct {
	Status  string `json:"status"`
	Service string `json:"service"`
}

func WriteJSON(w http.ResponseWriter, status int, body any) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(body)
}

func Mount(mux *http.ServeMux, service string) {
	mux.HandleFunc("/healthz", func(w http.ResponseWriter, _ *http.Request) {
		WriteJSON(w, http.StatusOK, Status{Status: "ok", Service: service})
	})
	mux.HandleFunc("/readyz", func(w http.ResponseWriter, _ *http.Request) {
		WriteJSON(w, http.StatusOK, Status{Status: "ready", Service: service})
	})
}
