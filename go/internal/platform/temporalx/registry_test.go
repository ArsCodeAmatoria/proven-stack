package temporalx_test

import (
	"testing"

	"github.com/ArsCodeAmatoria/proven-stack/go/internal/platform/temporalx"
)

func TestEmptyRegistries(t *testing.T) {
	workflows := temporalx.NewWorkflowRegistry()
	activities := temporalx.NewActivityRegistry()
	if workflows.Len() != 0 || activities.Len() != 0 {
		t.Fatalf("expected empty registries")
	}
	if err := workflows.Register(temporalx.WorkflowDefinition{
		Name: "Demo", TaskQueue: "proven-io", Description: "test",
	}); err != nil {
		t.Fatal(err)
	}
	if workflows.Len() != 1 {
		t.Fatalf("expected 1 workflow, got %d", workflows.Len())
	}
	if err := workflows.Register(temporalx.WorkflowDefinition{Name: "Demo"}); err == nil {
		t.Fatal("expected duplicate registration error")
	}
}
