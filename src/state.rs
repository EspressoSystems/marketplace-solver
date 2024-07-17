use std::sync::Arc;

use async_std::sync::RwLock;
use espresso_types::{NamespaceId, SeqTypes};
use hotshot_types::{data::ViewNumber, traits::node_implementation::NodeType};

// TODO ED: Implement a shared solver state with the HotShot events received
#[derive(Default, Clone)]
pub struct SolverState {}

pub trait UpdateSolverState {
    fn submit_bix_tx(&self);
    fn register_rollup(&self);
    fn update_rollup_registration(&self, namespace_id: NamespaceId);
    fn get_all_rollup_registrations(&self);
}

pub trait SolverStateDataSource {
    fn get_auction_results_permissionless(view_number: ViewNumber);
    fn get_auction_results_permissioned(
        view_number: ViewNumber,
        signauture: <SeqTypes as NodeType>::SignatureKey,
    );
}

impl UpdateSolverState for Arc<RwLock<SolverState>> {
    fn submit_bix_tx(&self) {}

    fn register_rollup(&self) {}

    fn update_rollup_registration(&self, _namespace_id: NamespaceId) {}

    fn get_all_rollup_registrations(&self) {}
}

impl SolverStateDataSource for SolverState {
    fn get_auction_results_permissionless(_view_number: ViewNumber) {}

    fn get_auction_results_permissioned(
        _view_number: ViewNumber,
        _signauture: <SeqTypes as NodeType>::SignatureKey,
    ) {
    }
}
