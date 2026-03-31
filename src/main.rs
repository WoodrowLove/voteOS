//! VoteOS — Binary entrypoint.
//!
//! Loads config, initializes registries, starts HTTP server.
//! All business logic lives in domain/ and workflows/.
//! This file is thin glue.

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

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "voteos=info,tower_http=info".into()),
        )
        .init();

    // Load config
    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "config/voteos.toml".into());

    let config_str = std::fs::read_to_string(&config_path)
        .unwrap_or_else(|e| {
            eprintln!("Failed to read config {}: {}", config_path, e);
            std::process::exit(1);
        });

    let config: Config = toml::from_str(&config_str)
        .unwrap_or_else(|e| {
            eprintln!("Failed to parse config: {}", e);
            std::process::exit(1);
        });

    // Validate config
    if config.security.require_auth && config.security.api_key.is_empty() {
        eprintln!("FATAL: require_auth=true but api_key is empty");
        std::process::exit(1);
    }

    // Initialize registries
    let data_dir = PathBuf::from(&config.persistence.data_dir);

    let state = if config.persistence.enabled {
        tracing::info!("Persistence enabled, data_dir: {}", config.persistence.data_dir);
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
            },
        })
    } else {
        tracing::info!("Persistence disabled, using in-memory storage");
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
            },
        })
    };

    let app = api::build_router(state);

    let bind = format!("{}:{}", config.api.bind_address, config.api.bind_port);
    tracing::info!("VoteOS starting on {}", bind);

    let listener = tokio::net::TcpListener::bind(&bind).await
        .unwrap_or_else(|e| {
            eprintln!("Failed to bind {}: {}", bind, e);
            std::process::exit(1);
        });

    axum::serve(listener, app).await
        .unwrap_or_else(|e| {
            eprintln!("Server error: {}", e);
            std::process::exit(1);
        });
}
