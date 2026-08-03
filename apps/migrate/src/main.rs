use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Context;
use clap::{Parser, Subcommand};
use proven_config::load;
use proven_db::{connect_pool, migrate, migrations_dir, run_seeds, seeds_dir};

#[derive(Parser, Debug)]
#[command(
    name = "proven-migrate",
    about = "Proven PostgreSQL migrate/seed CLI (no business schema)"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Apply pending sqlx migrations from db/migrations/platform
    Migrate {
        #[arg(long)]
        dir: Option<PathBuf>,
    },
    /// Run ordered *.sql seeds for a profile (local|ci)
    Seed {
        #[arg(default_value = "local")]
        profile: String,
        #[arg(long)]
        dir: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .compact()
        .init();

    if let Err(err) = run().await {
        eprintln!("error: {err:#}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = load().context("configuration load failed")?;
    let pool = connect_pool(&config)
        .await
        .context("database connection failed")?;

    match cli.command {
        Commands::Migrate { dir } => {
            let path = dir.unwrap_or_else(|| {
                if !config.database.migrations_dir.is_empty() {
                    PathBuf::from(&config.database.migrations_dir)
                } else {
                    migrations_dir()
                }
            });
            let status = migrate(&pool, &path).await.context("migrate failed")?;
            println!(
                "migrations ok: applied={} directory={}",
                status.applied, status.directory
            );
        }
        Commands::Seed { profile, dir } => {
            let path = dir.unwrap_or_else(|| seeds_dir(&profile));
            let ran = run_seeds(&pool, &path).await.context("seed failed")?;
            println!(
                "seeds ok: files_executed={ran} directory={}",
                path.display()
            );
        }
    }

    Ok(())
}
