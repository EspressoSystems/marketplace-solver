use std::{collections::HashMap, sync::Arc};

use async_std::sync::RwLock;
use async_trait::async_trait;
use espresso_types::{NamespaceId, PubKey, SeqTypes};
use hotshot_types::{
    data::ViewNumber, signature_key::BuilderKey, traits::node_implementation::NodeType, PeerConfig,
};

use crate::database::PostgresClient;

// TODO ED: Implement a shared solver state with the HotShot events received
pub struct GlobalState {
    pub solver: Arc<RwLock<SolverState>>,
    pub persistence: PostgresClient,
}

impl GlobalState {
    pub async fn new(db: PostgresClient, state: SolverState) -> anyhow::Result<Self> {
        Ok(Self {
            solver: Arc::new(RwLock::new(state)),
            persistence: db,
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
            solver: Arc::new(RwLock::new(SolverState::mock())),
            persistence: client,
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
