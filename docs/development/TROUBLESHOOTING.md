# Troubleshooting

| Symptom | Fix |
| --- | --- |
| `just: command not found` | `brew install just` or use `make` wrappers |
| Lefthook hooks missing | `just hooks` / `pnpm install` |
| Postgres migrate fails | `just deps` then retry `just db-migrate` |
| Web engine warning Node 20.7 | Upgrade to ≥ 20.19 |
| Better Auth users vanish | In-memory adapter — expected until Core DB |
| Clippy fails in hook | Run `cargo clippy --workspace --all-targets -- -D warnings` |
| Playwright can’t start | Ensure port 3000 free; set `PLAYWRIGHT_SKIP_WEBSERVER=1` if server already up |
| Arch Go check needs `rg` | Install ripgrep (`brew install ripgrep`) |
