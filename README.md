<p align="center">
  <img src="assets/brand/logo.svg" alt="Proven logo" width="96" height="96" />
</p>

<h1 align="center">Proven</h1>

<p align="center">
  <strong>Construction Compliance Operating System</strong> for contractors who need defensible proof that people, equipment, and work are compliant—every day, on every site.
</p>

Proven is built for General Contractors, Prime Contractors, Subcontractors, Crane Companies, Concrete Forming Companies, Civil Contractors, and Industrial Contractors across **Canada, the United States, Australia, and New Zealand**.

Mobile-first for workers. Desktop-first for supervisors, safety coordinators, project managers, and administrators.

> Logo mark adapted from Lucide [`fingerprint-pattern`](https://lucide.dev/icons/fingerprint-pattern) (ISC).

---

## What it is

Proven is not a forms app. It is one cohesive platform for:

- Projects & people
- Safety operations
- Equipment compliance
- Documents & digital signatures
- Training & competency
- COR audit readiness
- Workflows, notifications, analytics, and administration

---

## Repository status

This monorepo is in **foundation** phase: product, domain, system, repository, and UX architecture are documented. Application scaffolding comes next.

```text
proven-stack/
├── AGENTS.md          # Engineering agent constitution
├── assets/brand/      # Logo and brand marks
├── docs/
│   ├── PRD.md
│   ├── architecture/
│   │   ├── DOMAIN_MODEL.md
│   │   ├── SYSTEM_ARCHITECTURE.md
│   │   └── REPOSITORY_PLAN.md
│   └── ux/
│       └── UX_ARCHITECTURE.md
└── README.md
```

---

## Documentation

| Doc | Description |
| --- | --- |
| [AGENTS.md](./AGENTS.md) | Engineering principles, stack, and hard constraints |
| [PRD](./docs/PRD.md) | Product requirements |
| [Domain Model](./docs/architecture/DOMAIN_MODEL.md) | DDD bounded contexts and aggregates |
| [System Architecture](./docs/architecture/SYSTEM_ARCHITECTURE.md) | Runtime architecture and deployment |
| [Repository Plan](./docs/architecture/REPOSITORY_PLAN.md) | Monorepo structure, CI/CD, standards |
| [UX Architecture](./docs/ux/UX_ARCHITECTURE.md) | Information architecture and experience |

---

## Intended stack

| Layer | Technology |
| --- | --- |
| Web / PWA | Next.js, TypeScript, Tailwind, shadcn/ui |
| API | Rust, Axum (modular monolith) |
| Workers | Go |
| Workflows | Temporal |
| Events | NATS |
| Data | PostgreSQL, Redis (cache only), Cloudflare R2, ClickHouse |
| Edge / deploy | Cloudflare, Vercel, Fly.io, Docker, GitHub Actions |

Architecture style: **modular monolith**. Modules own their domains and integrate through public interfaces, events, and Temporal workflows—never by reaching into another module’s internals.

---

## Core principles

- Domain-driven, API-first, security-first, offline-first
- Business rules live in domain modules—not in React or Go workers
- Redis is never permanent storage
- Never bypass Temporal for durable business workflows
- Never bypass audit logging
- Prefer long-term maintainability over short-term convenience

---

## Contributing

1. Read [AGENTS.md](./AGENTS.md) and the architecture docs above.
2. Follow trunk-based development on short-lived branches into `main` (see [Repository Plan](./docs/architecture/REPOSITORY_PLAN.md)).
3. Keep module boundaries intact; cross-module work uses interfaces, events, or workflows only.

---

## License

License TBD.
