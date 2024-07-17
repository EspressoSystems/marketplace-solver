use std::sync::Arc;

use async_std::sync::RwLock;
use async_trait::async_trait;
use espresso_types::{NamespaceId, SeqTypes};
use hotshot_types::{data::ViewNumber, traits::node_implementation::NodeType};

// TODO ED: Implement a shared solver state with the HotShot events received
pub struct GlobalState {
    pub solver: Arc<RwLock<SolverState>>,
    // persistence: PostgresClient,
}

impl GlobalState {
    pub fn mock() -> Self {
        Self {
            solver: Arc::new(RwLock::new(SolverState)),
        }
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
