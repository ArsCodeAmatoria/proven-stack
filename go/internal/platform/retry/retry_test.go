package retry_test

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/ArsCodeAmatoria/proven-stack/go/internal/platform/retry"
)

func TestDoSucceedsAfterFailures(t *testing.T) {
	attempts := 0
	err := retry.Do(context.Background(), retry.Policy{
		MaxAttempts: 3,
		Initial:     time.Millisecond,
		Max:         5 * time.Millisecond,
		Multiplier:  2,
	}, func(context.Context) error {
		attempts++
		if attempts < 3 {
			return errors.New("transient")
		}
		return nil
	})
	if err != nil {
		t.Fatalf("Do: %v", err)
	}
	if attempts != 3 {
		t.Fatalf("attempts=%d", attempts)
	}
}

func TestDoRespectsContext(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	cancel()
	err := retry.Do(ctx, retry.Default(), func(context.Context) error {
		return errors.New("nope")
	})
	if !errors.Is(err, context.Canceled) {
		t.Fatalf("want canceled, got %v", err)
	}
}
