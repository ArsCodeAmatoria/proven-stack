# Security Policy

## Reporting a vulnerability

**Do not** open a public GitHub issue with exploit details, credentials, or customer data.

Report security vulnerabilities privately to the Proven security contact (email/team channel TBD for this organization). Include:

- Affected component (`web`, `api`, `workers`, contracts, infra)  
- Description and impact  
- Reproduction steps (minimal)  
- Whether the issue is already exploited (if known)  
- Your contact details for follow-up  

We will acknowledge receipt as soon as practical and coordinate fix + disclosure timing.

For architectural controls (AuthN/AuthZ, encryption, Cloudflare, OWASP alignment), see [Security Architecture](./docs/architecture/SECURITY_ARCHITECTURE.md).

---

## Supported versions

| Stream | Support |
| --- | --- |
| Latest released `vX.Y.Z` | Security fixes |
| `main` staging builds | Monitored; not production SLA |
| Older minor releases | Best-effort until policy expands |

---

## Safe harbor (good-faith research)

Good-faith security research that follows this policy and avoids privacy violations, data destruction, and service disruption is appreciated. Do not access data that is not yours; do not perform DoS against production.

---

## Secrets and supply chain

- Never commit secrets, API keys, or private keys. Use `.env.example` and platform secret stores.  
- Dependency updates are managed via Dependabot (see repository design); review majors carefully.  
- CI should include secret scanning and SAST (CodeQL or equivalent) once workflows are enabled.

---

## Security-sensitive changes

PRs touching authentication, authorization, tenancy isolation, cryptography, file upload/AV, guest signing, or PII export require extra review (`risk:security-sensitive`) and appropriate CODEOWNERS approval.
