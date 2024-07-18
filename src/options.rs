use std::{str::FromStr, time::Duration};

use clap::Parser;
use snafu::Snafu;
use tide_disco::Url;

use crate::database::PostgresClient;

#[derive(Parser, Clone, Debug)]
pub struct Options {
    #[clap(long, env = "HOTSHOT_EVENTS_API_URL")]
    pub events_url: Url,

    #[clap(flatten)]
    pub database_options: DatabaseOptions,
}

/// Arguments for establishing a database connection
#[derive(Clone, Debug, Parser)]
pub struct DatabaseOptions {
    // Postgres URL connection string
    #[clap(long, env = "MARKETPLACE_SOLVER_POSTGRES_URL")]
    pub url: Option<String>,

    #[clap(long, env = "MARKETPLACE_SOLVER_POSTGRES_HOST")]
    pub host: Option<String>,

    #[clap(long, env = "MARKETPLACE_SOLVER_POSTGRES_PORT")]
    pub port: Option<u16>,

    #[clap(long, env = "MARKETPLACE_SOLVER_POSTGRES_DATABASE_NAME")]
    pub db_name: Option<String>,

    #[clap(long, env = "MARKETPLACE_SOLVER_POSTGRES_USERNAME")]
    pub username: Option<String>,

    #[clap(long, env = "MARKETPLACE_SOLVER_POSTGRES_PASSWORD")]
    pub password: Option<String>,

    #[clap(long, env = "MARKETPLACE_SOLVER_POSTGRES_MAX_CONNECTIONS")]
    pub max_connections: Option<u32>,

    #[clap(long,value_parser = parse_duration, env = "MARKETPLACE_SOLVER_DATABASE_ACQUIRE_TIMEOUT")]
    pub acquire_timeout: Option<Duration>,

    #[clap(long,value_parser = parse_duration, env = "MARKETPLACE_SOLVER_DATABASE_REQUIRE_SSL", default_value_t = false)]
    pub require_ssl: bool,

    #[clap(long,value_parser = parse_duration, env = "MARKETPLACE_SOLVER_DATABASE_RUN_MIGRATIONS",  default_value_t = true)]
    pub migrations: bool,
}

impl DatabaseOptions {
    pub async fn connect(self) -> anyhow::Result<PostgresClient> {
        PostgresClient::connect(self).await
    }
}

#[derive(Clone, Debug, Snafu)]
pub struct ParseDurationError {
    reason: String,
}

pub fn parse_duration(s: &str) -> Result<Duration, ParseDurationError> {
    cld::ClDuration::from_str(s)
        .map(Duration::from)
        .map_err(|err| ParseDurationError {
            reason: err.to_string(),
        })
}
