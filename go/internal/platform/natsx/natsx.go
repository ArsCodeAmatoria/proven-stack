// Package natsx wraps NATS connectivity for Proven workers (I/O only).
package natsx

import (
	"fmt"
	"time"

	"github.com/nats-io/nats.go"
)

// Options configures the NATS connection.
type Options struct {
	URL     string
	Name    string
	Timeout time.Duration
}

// Connect opens a NATS connection with reconnect-friendly defaults.
func Connect(opts Options) (*nats.Conn, error) {
	if opts.URL == "" {
		return nil, fmt.Errorf("nats url is required")
	}
	if opts.Timeout <= 0 {
		opts.Timeout = 5 * time.Second
	}

	nc, err := nats.Connect(
		opts.URL,
		nats.Name(opts.Name),
		nats.Timeout(opts.Timeout),
		nats.MaxReconnects(-1),
		nats.ReconnectWait(time.Second),
		nats.PingInterval(20*time.Second),
		nats.DrainTimeout(10*time.Second),
	)
	if err != nil {
		return nil, fmt.Errorf("nats connect: %w", err)
	}
	return nc, nil
}

// Ready reports whether the connection is usable.
func Ready(nc *nats.Conn) bool {
	return nc != nil && nc.IsConnected()
}
