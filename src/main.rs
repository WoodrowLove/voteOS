//! VoteOS — Binary entrypoint.
//!
//! Loads config, validates environment, initializes registries, starts HTTP server.
//! All business logic lives in domain/ and workflows/.
//! This file is thin glue + startup validation.

use std::sync::Arc;
use std::path::PathBuf;

mod api;

#[derive(serde::Deserialize)]
struct Config {
    api: ApiSection,
    security: SecuritySection,
    persistence: PersistenceSection,
}

#[derive(serde::Deserialize)]
struct ApiSection {
    bind_address: String,
    bind_port: u16,
}

#[derive(serde::Deserialize)]
struct SecuritySection {
    api_key: String,
    require_auth: bool,
}

#[derive(serde::Deserialize)]
struct PersistenceSection {
    data_dir: String,
    enabled: bool,
}

/// Validate config at startup. Fail fast on invalid configuration.
fn validate_config(config: &Config) -> Result<(), String> {
    // Auth validation
    if config.security.require_auth && config.security.api_key.is_empty() {
        return Err("require_auth=true but api_key is empty".into());
    }
    if config.security.require_auth && config.security.api_key.len() < 8 {
        return Err("api_key must be at least 8 characters when auth is required".into());
    }

    // Bind address validation
    if config.api.bind_address.is_empty() {
        return Err("bind_address cannot be empty".into());
    }
    if config.api.bind_port == 0 {
        return Err("bind_port cannot be 0".into());
    }

    // Persistence validation
    if config.persistence.enabled && config.persistence.data_dir.is_empty() {
        return Err("persistence.enabled=true but data_dir is empty".into());
    }

    Ok(())
}

/// Ensure persistence directory exists and is writable.
fn ensure_data_dir(path: &PathBuf) -> Result<(), String> {
    if !path.exists() {
        std::fs::create_dir_all(path)
            .map_err(|e| format!("Failed to create data directory {}: {}", path.display(), e))?;
        tracing::info!("Created data directory: {}", path.display());
    }

    if !path.is_dir() {
        return Err(format!("{} exists but is not a directory", path.display()));
    }

    // Test writability
    let test_file = path.join(".voteos_write_test");
    std::fs::write(&test_file, "test")
        .map_err(|e| format!("Data directory {} is not writable: {}", path.display(), e))?;
    let _ = std::fs::remove_file(&test_file);

    Ok(())
}

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "voteos=info,tower_http=info".into()),
        )
        .init();

    tracing::info!("VoteOS v{} starting", env!("CARGO_PKG_VERSION"));

    // Load config
    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "config/voteos.toml".into());

    let config_str = std::fs::read_to_string(&config_path)
        .unwrap_or_else(|e| {
            eprintln!("FATAL: Failed to read config {}: {}", config_path, e);
            std::process::exit(1);
        });

    let config: Config = toml::from_str(&config_str)
        .unwrap_or_else(|e| {
            eprintln!("FATAL: Failed to parse config {}: {}", config_path, e);
            std::process::exit(1);
        });

    // Validate config
    if let Err(e) = validate_config(&config) {
        eprintln!("FATAL: Config validation failed: {}", e);
        std::process::exit(1);
    }

    // Initialize persistence
    let data_dir = PathBuf::from(&config.persistence.data_dir);

    if config.persistence.enabled {
        if let Err(e) = ensure_data_dir(&data_dir) {
            eprintln!("FATAL: {}", e);
            std::process::exit(1);
        }
        tracing::info!("Persistence enabled, data_dir: {}", data_dir.display());
    } else {
        tracing::warn!("Persistence DISABLED — state will be lost on restart");
    }

    // Initialize registries
    let state = if config.persistence.enabled {
        Arc::new(api::AppState {
            voter_registry: voteos::domain::voters::VoterRegistry::with_data_dir(&data_dir),
            election_registry: voteos::domain::elections::ElectionRegistry::with_data_dir(&data_dir),
            ballot_registry: voteos::domain::ballots::BallotRegistry::with_data_dir(&data_dir),
            vote_registry: voteos::domain::votes::VoteRegistry::with_data_dir(&data_dir),
            tally_registry: voteos::domain::tally::TallyRegistry::with_data_dir(&data_dir),
            cert_registry: voteos::domain::certification::CertificationRegistry::with_data_dir(&data_dir),
            proposal_registry: voteos::domain::proposals::ProposalRegistry::with_data_dir(&data_dir),
            audit_registry: voteos::domain::audit::AuditRegistry::with_data_dir(&data_dir),
            export_registry: voteos::domain::export::ExportRegistry::with_data_dir(&data_dir),
            ops_registry: voteos::domain::operations::OperationsRegistry::with_data_dir(&data_dir),
            config: api::ApiConfig {
                api_key: config.security.api_key,
                require_auth: config.security.require_auth,
                persistence_enabled: true,
            },
        })
    } else {
        Arc::new(api::AppState {
            voter_registry: voteos::domain::voters::VoterRegistry::new(),
            election_registry: voteos::domain::elections::ElectionRegistry::new(),
            ballot_registry: voteos::domain::ballots::BallotRegistry::new(),
            vote_registry: voteos::domain::votes::VoteRegistry::new(),
            tally_registry: voteos::domain::tally::TallyRegistry::new(),
            cert_registry: voteos::domain::certification::CertificationRegistry::new(),
            proposal_registry: voteos::domain::proposals::ProposalRegistry::new(),
            audit_registry: voteos::domain::audit::AuditRegistry::new(),
            export_registry: voteos::domain::export::ExportRegistry::new(),
            ops_registry: voteos::domain::operations::OperationsRegistry::new(),
            config: api::ApiConfig {
                api_key: config.security.api_key,
                require_auth: config.security.require_auth,
                persistence_enabled: false,
            },
        })
    };

    // Log startup state
    tracing::info!(
        "Registries initialized — elections: {}, certifications: {}, auth: {}",
        state.election_registry.elections.count(),
        state.cert_registry.certifications.count(),
        if state.config.require_auth { "enabled" } else { "disabled" },
    );

    let app = api::build_router(state);

    let bind = format!("{}:{}", config.api.bind_address, config.api.bind_port);
    tracing::info!("VoteOS ready on http://{}", bind);

    let listener = tokio::net::TcpListener::bind(&bind).await
        .unwrap_or_else(|e| {
            eprintln!("FATAL: Failed to bind {}: {}", bind, e);
            std::process::exit(1);
        });

    axum::serve(listener, app).await
        .unwrap_or_else(|e| {
            eprintln!("FATAL: Server error: {}", e);
            std::process::exit(1);
        });
}
