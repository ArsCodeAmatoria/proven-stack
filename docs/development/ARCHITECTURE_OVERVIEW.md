# Architecture Overview

Proven is a **modular monolith**:

- **Rust** owns domain authority (future `proven-*` modules).
- **Next.js** is UX + Better Auth AuthN edge.
- **Go** workers are I/O-only (NATS/Temporal/providers).
- Integrate via public APIs, events, Temporal — never another module’s internals.

Companion: [`docs/architecture/SYSTEM_ARCHITECTURE.md`](../architecture/SYSTEM_ARCHITECTURE.md), [`AGENTS.md`](../../AGENTS.md).
