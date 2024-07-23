use std::collections::HashMap;

use crate::{database::PostgresClient, types::RollupRegistration, SolverError, SolverResult};
use async_trait::async_trait;
use espresso_types::{NamespaceId, PubKey, SeqTypes};
use hotshot_types::{
    data::ViewNumber, signature_key::BuilderKey, traits::node_implementation::NodeType, PeerConfig,
};
use serde::Deserialize;
use serde::Serialize;
use sqlx::{FromRow, PgPool};

// TODO ED: Implement a shared solver state with the HotShot events received
pub struct GlobalState {
    solver: SolverState,
    database: PostgresClient,
}

impl GlobalState {
    pub fn solver(&self) -> &SolverState {
        &self.solver
    }

    pub fn database(&self) -> &PgPool {
        self.database.pool()
    }
}

impl GlobalState {
    pub fn new(db: PostgresClient, state: SolverState) -> anyhow::Result<Self> {
        Ok(Self {
            solver: state,
            database: db,
        })
    }
}

pub struct SolverState {
    pub stake_table: StakeTable,
    // todo (abdul) : () will be replaced w BidTx
    pub bid_txs: HashMap<ViewNumber, HashMap<BuilderKey, ()>>,
}

pub struct StakeTable {
    pub known_nodes_with_stake: Vec<PeerConfig<PubKey>>,
}

#[async_trait]
pub trait UpdateSolverState {
    // TODO (abdul) : add BidTx from types crate.
    async fn submit_bix_tx(&mut self) -> anyhow::Result<()>;
    // TODO (abdul)
    async fn register_rollup(&self, registration: RollupRegistration) -> SolverResult<()>;
    async fn update_rollup_registration(&self, namespace_id: NamespaceId);
    async fn get_all_rollup_registrations(&self);
    // TODO (abdul) : return AuctionResults
    async fn calculate_auction_results_permissionless(_view_number: ViewNumber);
    async fn calculate_auction_results_permissioned(
        _view_number: ViewNumber,
        _signauture: <SeqTypes as NodeType>::SignatureKey,
    );
}

#[async_trait]
impl UpdateSolverState for GlobalState {
    async fn submit_bix_tx(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    async fn register_rollup(&self, registration: RollupRegistration) -> Result<(), SolverError> {
        let signature = &registration.signature;
        let signatures = &registration.signature_keys;
        if !signatures.contains(signature) {
            return Err(SolverError::InvalidSignature(signature.to_string()));
        };

        let db = self.database();

        let namespace_id = registration.namespace_id;

        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM rollup_registrations where namespace_id = $1;",
        )
        .bind(u64::from(namespace_id) as i64)
        .fetch_one(db)
        .await
        .map_err(SolverError::from)?;

        if exists {
            return Err(SolverError::RollupAlreadyExists(namespace_id));
        }

        let bytes = bincode::serialize(&registration).map_err(SolverError::from)?;

        let result = sqlx::query("INSERT INTO rollup_registrations VALUES ($1, $2);")
            .bind(u64::from(namespace_id) as i64)
            .bind(&bytes)
            .execute(db)
            .await
            .map_err(SolverError::from)?;

        if !result.rows_affected() != 1 {
            return Err(SolverError::Database(format!(
                "invalid num of rows affected. rows affected: {:?}",
                result.rows_affected()
            )));
        }

        Ok(())
    }

    async fn update_rollup_registration(&self, _namespace_id: NamespaceId) {}

    async fn get_all_rollup_registrations(&self) -> SolverResult<Vec<RollupRegistration>> {
        let db = self.database();

        let result: Vec<RollupRegistrationResult> =
            sqlx::query_as("SELECT data from rollup_registrations;")
                .fetch_all(db)
                .await
                .map_err(SolverError::from)?;
    }

    async fn calculate_auction_results_permissionless(_view_number: ViewNumber) {}
    async fn calculate_auction_results_permissioned(
        _view_number: ViewNumber,
        _signauture: <SeqTypes as NodeType>::SignatureKey,
    ) {
    }
}

#[derive(Debug, Deserialize, Serialize, FromRow)]
struct RollupRegistrationResult {
    namespace_id: i64,
    data: Vec<u8>,
}



#[cfg(any(test, feature = "testing"))]
impl GlobalState {
    pub async fn mock() -> Self {
        let db = hotshot_query_service::data_source::sql::testing::TmpDb::init().await;
        let host = db.host();
        let port = db.port();

        let opts = crate::DatabaseOptions {
            url: None,
            host: Some(host),
            port: Some(port),
            db_name: None,
            username: Some("postgres".to_string()),
            password: Some("password".to_string()),
            max_connections: Some(100),
            acquire_timeout: None,
            require_ssl: false,
            migrations: true,
        };

        let client = PostgresClient::connect(opts)
            .await
            .expect("failed to connect to database");

        Self {
            solver: SolverState::mock(),
            database: client,
        }
    }
}

#[cfg(any(test, feature = "testing"))]
impl SolverState {
    pub fn mock() -> Self {
        Self {
            stake_table: StakeTable {
                known_nodes_with_stake: crate::mock::generate_stake_table(),
            },
            bid_txs: Default::default(),
        }
    }
}
