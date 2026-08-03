# Proven developer handbook

Canonical onboarding and engineering standards for Proven (DX-1).

Architecture deep-dives remain under [`docs/architecture/`](../architecture/). Process/CI details also live in [`docs/engineering/`](../engineering/).

## Contents

| Guide | Topic |
| --- | --- |
| [Getting Started](./GETTING_STARTED.md) | Clone → `just setup` |
| [Local Development](./LOCAL_DEVELOPMENT.md) | Day-to-day commands |
| [Repository Structure](./REPOSITORY_STRUCTURE.md) | Monorepo map |
| [Coding Standards](./CODING_STANDARDS.md) | Shared expectations |
| [Commit Conventions](./COMMIT_CONVENTIONS.md) | Conventional Commits |
| [Branch Strategy](./BRANCH_STRATEGY.md) | Trunk-based flow |
| [Pull Request Process](./PULL_REQUEST_PROCESS.md) | Reviews & checks |
| [Architecture Overview](./ARCHITECTURE_OVERVIEW.md) | Modular monolith |
| [DDD Guidelines](./DDD_GUIDELINES.md) | Bounded contexts |
| [Architecture Gates](./ARCHITECTURE_GATES.md) | Automated boundaries |
| [Rust Guidelines](./RUST_GUIDELINES.md) | API / crates |
| [Go Guidelines](./GO_GUIDELINES.md) | Workers (I/O only) |
| [TypeScript Guidelines](./TYPESCRIPT_GUIDELINES.md) | Next.js / packages |
| [Database Guidelines](./DATABASE_GUIDELINES.md) | Migrations & seeds |
| [Seeds & Fixtures](./SEEDS_AND_FIXTURES.md) | Demo data (no business schema yet) |
| [Testing Guide](./TESTING_GUIDE.md) | Unit / integration / e2e |
| [Debugging Guide](./DEBUGGING_GUIDE.md) | Local troubleshooting |
| [API Documentation](./API_DOCUMENTATION.md) | OpenAPI / Swagger / Redoc |
| [Release Process](./RELEASE_PROCESS.md) | release-please |
| [CI/CD](./CI_CD.md) | GitHub Actions |
| [Security Practices](./SECURITY_PRACTICES.md) | Secrets & hooks |
| [Editor Setup](./EDITOR_SETUP.md) | VS Code / Dev Containers |
| [Engineering Metrics](./ENGINEERING_METRICS.md) | Quality signals |
| [Troubleshooting](./TROUBLESHOOTING.md) | Common failures |
| [FAQ](./FAQ.md) | Quick answers |

**Task runner:** [`just`](../../justfile) is the source of truth (`Makefile` wraps it).
