// Package retry provides shared backoff policies for I/O workers.
// No domain / business logic.
package retry

import (
	"context"
	"errors"
	"math"
	"math/rand"
	"time"
)

// Policy describes exponential backoff with jitter and a max attempt count.
type Policy struct {
	MaxAttempts int
	Initial     time.Duration
	Max         time.Duration
	Multiplier  float64
	Jitter      float64 // 0..1 fraction of delay
}

// Default is suitable for transient I/O (NATS publish, HTTP callbacks).
func Default() Policy {
	return Policy{
		MaxAttempts: 5,
		Initial:     200 * time.Millisecond,
		Max:         10 * time.Second,
		Multiplier:  2,
		Jitter:      0.2,
	}
}

// TemporalActivity mirrors Temporal's typical activity retry envelope (documentation helper).
func TemporalActivity() Policy {
	return Policy{
		MaxAttempts: 10,
		Initial:     time.Second,
		Max:         60 * time.Second,
		Multiplier:  2,
		Jitter:      0.1,
	}
}

// Do executes fn until success, context cancel, or attempts exhausted.
func Do(ctx context.Context, p Policy, fn func(context.Context) error) error {
	if p.MaxAttempts < 1 {
		p.MaxAttempts = 1
	}
	if p.Initial <= 0 {
		p.Initial = 100 * time.Millisecond
	}
	if p.Max <= 0 {
		p.Max = 30 * time.Second
	}
	if p.Multiplier < 1 {
		p.Multiplier = 2
	}

	var last error
	delay := p.Initial
	for attempt := 1; attempt <= p.MaxAttempts; attempt++ {
		if err := ctx.Err(); err != nil {
			return err
		}
		last = fn(ctx)
		if last == nil {
			return nil
		}
		if attempt == p.MaxAttempts {
			break
		}
		wait := withJitter(delay, p.Jitter)
		timer := time.NewTimer(wait)
		select {
		case <-ctx.Done():
			timer.Stop()
			return ctx.Err()
		case <-timer.C:
		}
		next := time.Duration(float64(delay) * p.Multiplier)
		if next > p.Max {
			next = p.Max
		}
		delay = next
	}
	if last == nil {
		return errors.New("retry exhausted")
	}
	return last
}

func withJitter(base time.Duration, jitter float64) time.Duration {
	if jitter <= 0 {
		return base
	}
	if jitter > 1 {
		jitter = 1
	}
	delta := float64(base) * jitter
	n := rand.Float64()*2*delta - delta
	out := float64(base) + n
	if out < 0 {
		return 0
	}
	return time.Duration(math.Round(out))
}
