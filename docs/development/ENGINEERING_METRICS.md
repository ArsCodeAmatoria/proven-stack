# Engineering Metrics

Track engineering health (DX-1 design; tooling hooks where noted).

| Metric | Source |
| --- | --- |
| Build / test duration | GitHub Actions job timings |
| Coverage | Codecov (`codecov.yml`) + `just test-coverage` |
| Dependency health | Dependabot PRs + `pnpm audit` / `cargo deny` (future) |
| Open vulnerabilities | GitHub Dependabot alerts |
| Tech debt markers | `rg 'TODO|FIXME'` (advisory) |
| PR cycle time | GitHub Insights |
| Deployment frequency / MTTR | N/A until deploy pipelines exist |

Prefer improving signal quality over vanity dashboards.
