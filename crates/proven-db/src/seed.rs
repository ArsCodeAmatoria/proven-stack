use std::fs;
use std::path::{Path, PathBuf};

use sqlx::PgPool;

use crate::DbError;

/// Resolve seed directory for an environment profile (`local` | `ci`).
pub fn seeds_dir(profile: &str) -> PathBuf {
    if let Ok(dir) = std::env::var("PROVEN_SEEDS_DIR") {
        if !dir.trim().is_empty() {
            return PathBuf::from(dir);
        }
    }
    PathBuf::from(format!("db/seeds/{profile}"))
}

/// Execute ordered `*.sql` seed files. Empty directories are a no-op success.
/// Seeds must not create business schema — foundation profiles are empty.
pub async fn run_seeds(pool: &PgPool, dir: impl AsRef<Path>) -> Result<usize, DbError> {
    let dir = dir.as_ref();
    if !dir.exists() {
        tracing::info!(directory = %dir.display(), "seed directory missing; skipping");
        return Ok(0);
    }

    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|e| DbError::Seed(e.to_string()))?
        .filter_map(|entry| entry.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("sql"))
        })
        .collect();
    files.sort();

    let mut ran = 0usize;
    for path in files {
        let sql = fs::read_to_string(&path).map_err(|e| DbError::Seed(e.to_string()))?;
        let trimmed = sql.trim();
        if trimmed.is_empty() || trimmed.lines().all(|l| {
            let t = l.trim();
            t.is_empty() || t.starts_with("--")
        }) {
            tracing::debug!(file = %path.display(), "skipping empty/comment-only seed");
            continue;
        }

        sqlx::raw_sql(&sql)
            .execute(pool)
            .await
            .map_err(|e| DbError::Seed(format!("{}: {e}", path.display())))?;
        ran += 1;
        tracing::info!(file = %path.display(), "seed applied");
    }

    Ok(ran)
}
