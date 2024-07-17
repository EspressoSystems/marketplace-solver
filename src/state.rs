use std::sync::Arc;

use anyhow::Context;
use async_std::sync::RwLock;
use async_trait::async_trait;
use espresso_types::{NamespaceId, SeqTypes};
use hotshot_types::{data::ViewNumber, traits::node_implementation::NodeType};

use crate::{database::PostgresClient, DatabaseOptions};

// TODO ED: Implement a shared solver state with the HotShot events received
pub struct GlobalState {
    pub solver: Arc<RwLock<SolverState>>,
    pub persistence: PostgresClient,
}

impl GlobalState {
    pub async fn new(opts: DatabaseOptions) -> anyhow::Result<Self> {
        Ok(Self {
            solver: Arc::new(RwLock::new(SolverState)),
            persistence: opts
                .connect()
                .await
                .context("failed to connect to database")?,
        })
    }
}

pub struct SolverState;

#[async_trait]
pub trait UpdateSolverState {
    // TODO (abdul) : add BidTx from types crate once https://github.com/EspressoSystems/espresso-sequencer/pull/1696 is merged
    async fn submit_bix_tx(&mut self) -> anyhow::Result<()>;
    // TODO (abdul)
    async fn register_rollup(&self);
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

    async fn register_rollup(&self) {}

    async fn update_rollup_registration(&self, _namespace_id: NamespaceId) {}

    async fn get_all_rollup_registrations(&self) {}

    async fn calculate_auction_results_permissionless(_view_number: ViewNumber) {}
    async fn calculate_auction_results_permissioned(
        _view_number: ViewNumber,
        _signauture: <SeqTypes as NodeType>::SignatureKey,
    ) {
    }
}

#[cfg(test)]
impl GlobalState {
    pub async fn mock() -> Self {
        let db = hotshot_query_service::data_source::sql::testing::TmpDb::init().await;
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

        Self {
            solver: Arc::new(RwLock::new(SolverState)),
            persistence: client,
        }
    }
}
