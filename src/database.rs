use sqlx::{pool::PoolConnection, postgres::PgPoolOptions, Error, PgPool, Postgres};

use crate::DatabaseOptions;

pub struct PostgresClient(PgPool);

impl PostgresClient {
    pub async fn connect(opts: DatabaseOptions) -> anyhow::Result<Self> {
        let DatabaseOptions {
            url,
            max_connections,
            acquire_timeout,
            migrations,
        } = opts;

        let mut options = PgPoolOptions::new();

        if let Some(max_connections) = max_connections {
            options = options.max_connections(max_connections);
        }

        if let Some(acquire_timeout) = acquire_timeout {
            options = options.acquire_timeout(acquire_timeout);
        }

        let connection = options.connect(&url).await?;

        if migrations {
            sqlx::migrate!("./migration").run(&connection).await?;
        }

        Ok(Self(connection))
    }

    pub async fn acquire(&self) -> Result<PoolConnection<Postgres>, Error> {
        self.0.acquire().await
    }
}
