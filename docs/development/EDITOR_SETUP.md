# Editor Setup

## VS Code

Workspace files: [`.vscode/`](../../.vscode/).

Recommended extensions are listed in `extensions.json` (Rust Analyzer, Go, ESLint, Prettier, Tailwind, Docker, Playwright, Vitest, Lefthook, Error Lens).

### Format on save

Enabled for Rust (rustfmt), Go (gofmt), TypeScript (Prettier).

### Fonts / themes (recommended, not enforced)

- Fonts: JetBrains Mono or Cascadia Code
- Themes: GitHub Dark / Light or One Dark Pro

## Dev Containers

[`.devcontainer/devcontainer.json`](../../.devcontainer/devcontainer.json) installs Node/Rust/Go + Docker-in-Docker and runs setup on create.
