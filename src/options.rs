use std::{str::FromStr, time::Duration};

use clap::Parser;
use snafu::Snafu;
use tide_disco::Url;

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
    #[clap(long, env = "MARKETPLACE_SOLVER_DATABASE_URL")]
    pub url: String,

    #[clap(long, env = "MARKETPLACE_SOLVER_DATABASE_MAX_CONNECTIONS")]
    pub max_connections: Option<u32>,

    #[clap(long,value_parser = parse_duration, env = "MARKETPLACE_SOLVER_DATABASE_ACQUIRE_TIMEOUT")]
    pub acquire_timeout: Option<Duration>,

    #[clap(long,value_parser = parse_duration, env = "MARKETPLACE_SOLVER_DATABASE_RUN_MIGRATIONS",  default_value_t = true)]
    pub migrations: bool,
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
