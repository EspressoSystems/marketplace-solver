use anyhow::Context;
use sqlx::{
    pool::PoolConnection,
    postgres::{PgConnectOptions, PgPoolOptions},
    ConnectOptions, Error, PgPool, Postgres,
};
use tide_disco::Url;

use crate::DatabaseOptions;

pub struct PostgresClient(PgPool);

impl PostgresClient {
    pub async fn connect(opts: DatabaseOptions) -> anyhow::Result<Self> {
        let DatabaseOptions {
            url,
            host,
            port,
            db_name,
            username,
            password,
            max_connections,
            acquire_timeout,
            migrations,
        } = opts;

        let mut options = PgPoolOptions::new();

        let postgres_url: Url = match url {
            Some(url) => url.parse()?,
            None => {
                let host = host.context("host not provided")?;
                let port = port.context("port not provided")?;
                let mut connect_opts = PgConnectOptions::new().host(&host).port(port);

                if let Some(username) = username {
                    connect_opts = connect_opts.username(&username);
                }

                if let Some(password) = password {
                    connect_opts = connect_opts.password(&password);
                }

                if let Some(db_name) = db_name {
                    connect_opts = connect_opts.database(&db_name);
                }

                connect_opts.to_url_lossy()
            }
        };

        if let Some(max_connections) = max_connections {
            options = options.max_connections(max_connections);
        }

        if let Some(acquire_timeout) = acquire_timeout {
            options = options.acquire_timeout(acquire_timeout);
        }

        let connection = options.connect(postgres_url.as_str()).await?;

        if migrations {
            sqlx::migrate!("./migration").run(&connection).await?;
        }

        Ok(Self(connection))
    }

    pub fn pool(&self) -> &PgPool {
        &self.0
    }

    pub async fn acquire(&self) -> Result<PoolConnection<Postgres>, Error> {
        self.0.acquire().await
    }
}

#[cfg(all(test, not(target_os = "windows")))]
mod test {
    use hotshot_query_service::data_source::sql::testing::TmpDb;

    use super::PostgresClient;
    use crate::DatabaseOptions;

    #[async_std::test]
    async fn test_database_connection() {
        let db = TmpDb::init().await;
        let host = db.host();
        let port = db.port();

        let opts = DatabaseOptions {
            url: None,
            host: Some(host),
            port: Some(port),
            db_name: None,
            username: Some("postgres".to_string()),
            password: Some("password".to_string()),
            max_connections: Some(100),
            acquire_timeout: None,
            migrations: true,
        };

        let client = PostgresClient::connect(opts)
            .await
            .expect("failed to connect to database");

        let pool = client.pool();

        sqlx::query("INSERT INTO test (str) VALUES ('testing');")
            .execute(pool)
            .await
            .unwrap();

        let result: i64 = sqlx::query_scalar("Select id from test where str = 'testing';")
            .fetch_one(pool)
            .await
            .unwrap();

        assert_eq!(result, 1);
    }
}
