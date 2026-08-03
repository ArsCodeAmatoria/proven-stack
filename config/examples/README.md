# Configuration examples

| File | Environment | Notes |
| --- | --- | --- |
| [`development.env`](./development.env) | `development` | Local defaults; weak secrets allowed |
| [`testing.env`](./testing.env) | `testing` | CI / integration; no `proven:proven` DB user |
| [`production.env`](./production.env) | `production` | Placeholders only — inject real secrets at deploy |

Root template: [`.env.example`](../../.env.example)

Loaders fail fast on missing keys and weak production secrets. See [Environment Configuration](../../docs/engineering/ENVIRONMENT_CONFIGURATION.md).
