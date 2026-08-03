#!/usr/bin/env python3
"""Fail if Rust workspace packages depend on forbidden targets."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

# Foundation allowlist: apps may depend on these crates only (plus workspace members that are infra).
ALLOWED_APP_DEPS = {
    "proven-api": {"proven-platform", "proven-config", "proven-shared"},
    "proven-migrate": {"proven-config", "proven-db"},
}

# Crates that must never depend on each other as "feature modules" (future).
FEATURE_PREFIX = "proven-"
INFRA = {
    "proven-shared",
    "proven-platform",
    "proven-config",
    "proven-db",
    "proven-observability",
}


def main() -> int:
    proc = subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--no-deps"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    if proc.returncode != 0:
        print(proc.stderr)
        return 1

    meta = json.loads(proc.stdout)
    packages = {p["name"]: p for p in meta["packages"] if p["id"].startswith(str(ROOT)) or "proven" in p["name"]}

    # Prefer workspace members from metadata
    members = []
    for p in meta["packages"]:
        manifest = Path(p["manifest_path"])
        try:
            manifest.relative_to(ROOT)
        except ValueError:
            continue
        members.append(p)

    fail = 0
    for pkg in members:
        name = pkg["name"]
        dep_names = {d["name"] for d in pkg.get("dependencies", [])}

        if name in ALLOWED_APP_DEPS:
            unexpected = {
                d
                for d in dep_names
                if d.startswith("proven-") and d not in ALLOWED_APP_DEPS[name] and d != name
            }
            # Allow transitive infra listed in Cargo.toml explicitly only
            # Filter to workspace proven-* only
            workspace_proven = {p["name"] for p in members}
            unexpected &= workspace_proven
            if unexpected:
                print(f"error: {name} has unexpected proven-* deps: {sorted(unexpected)}")
                fail = 1

        # Feature modules must not import other feature modules (when they exist).
        if name.startswith(FEATURE_PREFIX) and name not in INFRA and name not in (
            "proven-api",
            "proven-migrate",
        ):
            for d in dep_names:
                if (
                    d.startswith(FEATURE_PREFIX)
                    and d not in INFRA
                    and d != name
                    and d not in ("proven-api", "proven-migrate")
                ):
                    print(f"error: feature crate {name} must not depend on feature crate {d}")
                    fail = 1

    # Forbid apps importing future modules path via path deps under crates/modules
    for pkg in members:
        for d in pkg.get("dependencies", []):
            path = d.get("path")
            if path and "crates/modules" in path.replace("\\", "/"):
                print(f"error: {pkg['name']} depends on crates/modules path {path}")
                fail = 1

    return fail


if __name__ == "__main__":
    sys.exit(main())
