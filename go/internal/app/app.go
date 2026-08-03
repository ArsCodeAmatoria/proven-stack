package app

import (
	"log"
	"net/http"

	"github.com/ArsCodeAmatoria/proven-stack/go/internal/config"
	"github.com/ArsCodeAmatoria/proven-stack/go/internal/platform/health"
)

func Run(service, defaultPort string) {
	cfg := config.Load(service, defaultPort)
	mux := http.NewServeMux()
	health.Mount(mux, cfg.Service)

	mux.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/" {
			http.NotFound(w, r)
			return
		}
		health.WriteJSON(w, http.StatusOK, map[string]string{
			"service": cfg.Service,
			"status":  "running",
			"mode":    "foundation",
		})
	})

	log.Printf("%s listening on %s", cfg.Service, cfg.Addr())
	if err := http.ListenAndServe(cfg.Addr(), mux); err != nil {
		log.Fatal(err)
	}
}
