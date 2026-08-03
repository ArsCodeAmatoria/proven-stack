# Local Development

```bash
just --list
```

| Command | Purpose |
| --- | --- |
| `just up` / `just down` | Full Docker stack |
| `just deps` | Postgres/Redis/NATS/Temporal only |
| `just api` / `just web` / `just worker notify` | Host processes |
| `just dev` | Concurrent api+web+worker |
| `just fmt` / `just lint` / `just check` | Quality |
| `just test-fast` / `just test` | Tests |
| `just db-migrate` / `just db-seed` / `just db-reset` | Database |
| `just hooks` | Install Lefthook |
| `just arch` | Architecture gates |
| `just docs` | Export OpenAPI + pointers |

Env templates: [`.env.example`](../../.env.example), [`docs/engineering/ENVIRONMENT_CONFIGURATION.md`](../engineering/ENVIRONMENT_CONFIGURATION.md).

Docker details: [`docs/engineering/DOCKER_LOCAL_DEVELOPMENT.md`](../engineering/DOCKER_LOCAL_DEVELOPMENT.md).
