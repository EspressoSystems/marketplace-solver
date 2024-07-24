#![cfg(all(any(test, feature = "testing"), not(target_os = "windows")))]
#![allow(dead_code)]
use std::sync::Arc;

use async_compatibility_layer::art::async_spawn;
use async_std::{sync::RwLock, task::JoinHandle};
use espresso_types::SeqTypes;
use hotshot_query_service::data_source::sql::testing::TmpDb;
use hotshot_types::traits::node_implementation::NodeType;
use portpicker::pick_unused_port;
use tide_disco::{App, Url};
use vbs::version::StaticVersionType;

use crate::{
    database::{mock::setup_mock_database, PostgresClient},
    define_api, handle_events,
    mock::run_mock_event_service,
    state::{GlobalState, SolverState, StakeTable},
    EventsServiceClient, SolverError,
};

pub struct MockSolver {
    pub events_api: Url,
    pub solver_api: Url,
    pub state: Arc<RwLock<GlobalState>>,
    pub database: PostgresClient,
    pub handles: Vec<JoinHandle<()>>,
    pub tmp_db: TmpDb,
}

impl MockSolver {
    pub fn solver_api(&self) -> Url {
        self.solver_api.clone()
    }

    pub fn state(&self) -> Arc<RwLock<GlobalState>> {
        self.state.clone()
    }
}

impl Drop for MockSolver {
    fn drop(&mut self) {
        println!("canceling handles");

        while let Some(handle) = self.handles.pop() {
            async_std::task::block_on(handle.cancel());
        }
    }
}

impl MockSolver {
    pub async fn init() -> Self {
        let (tmp_db, database) = setup_mock_database().await;
        let (url, event_api_handle, generate_events_handle) = run_mock_event_service();

        let client = EventsServiceClient::new(url.clone()).await;
        let startup_info = client.get_startup_info().await.unwrap();
        let stream = client.get_event_stream().await.unwrap();

        let solver_state = SolverState {
            stake_table: StakeTable {
                known_nodes_with_stake: startup_info.known_node_with_stake,
            },
            bid_txs: Default::default(),
        };

        let state = Arc::new(RwLock::new(
            GlobalState::new(database.clone(), solver_state).unwrap(),
        ));

        let event_handler_handle = async_spawn({
            let state = state.clone();
            async move {
                let _ = handle_events(stream, state).await;
            }
        });

        let mut app = App::<_, SolverError>::with_state(state.clone());
        app.with_version(env!("CARGO_PKG_VERSION").parse().unwrap());

        let mut api = define_api(Default::default()).unwrap();
        api.with_version(env!("CARGO_PKG_VERSION").parse().unwrap());

        app.register_module::<SolverError, <SeqTypes as NodeType>::Base>("solver_api", api)
            .unwrap();

        let solver_api_port = pick_unused_port().expect("no free port");
        let solver_url: Url = Url::parse(&format!("http://localhost:{solver_api_port}")).unwrap();

        let solver_api_handle = async_spawn({
            let solver_url = solver_url.clone();
            async move {
                let _ = app
                    .serve(solver_url, <SeqTypes as NodeType>::Base::instance())
                    .await;
            }
        });

        let solver_api = solver_url.join("solver_api").unwrap();

        let handles = vec![
            generate_events_handle,
            event_handler_handle,
            event_api_handle,
            solver_api_handle,
        ];

        MockSolver {
            events_api: url,
            solver_api,
            state,
            database,
            tmp_db,
            handles,
        }
    }
}

#[cfg(test)]
mod test {

    use espresso_types::SeqTypes;
    use hotshot::types::{BLSPubKey, SignatureKey};
    use hotshot_types::traits::node_implementation::NodeType;

    use crate::{
        testing::MockSolver,
        types::{RollupRegistration, RollupUpdate},
        SolverError,
    };

    #[async_std::test]
    async fn test_rollup_registration() {
        let mock_solver = MockSolver::init().await;
        let solver_api = mock_solver.solver_api();
        let client =
            surf_disco::Client::<SolverError, <SeqTypes as NodeType>::Base>::new(solver_api);

        // Create a list of signature keys for rollup registration data
        let mut signature_keys = Vec::new();
        for _ in 0..10 {
            let private_key =
                <BLSPubKey as SignatureKey>::PrivateKey::generate(&mut rand::thread_rng());
            signature_keys.push(BLSPubKey::from_private(&private_key))
        }

        // the signature over the data should be from the list of signature keys
        let signature = signature_keys[0];

        // Initialize a rollup registration with namespace id = 1
        let mut reg_ns_1 = RollupRegistration {
            namespace_id: 1_u64.into(),
            reserve_price: 200.into(),
            active: true,
            signature_keys,
            text: "test".to_string(),
            signature,
        };

        // registering a rollup
        let result: RollupRegistration = client
            .post("register_rollup")
            .body_json(&reg_ns_1)
            .unwrap()
            .send()
            .await
            .unwrap();

        // Ensure the registration result matches the initial registration data
        assert_eq!(reg_ns_1, result);

        // Attempt to re-register the same rollup (should fail)
        let err = client
            .post::<RollupRegistration>("register_rollup")
            .body_json(&reg_ns_1)
            .unwrap()
            .send()
            .await
            .unwrap_err();

        // Ensure the error indicates the rollup with namespace id  = 1 already exists
        match err {
            SolverError::RollupAlreadyExists(id) if reg_ns_1.namespace_id == id => (),
            _ => panic!("err {err:?}"),
        }

        // Attempt to register a new rollup with namespace id = 2 using an invalid signature
        let private_key =
            <BLSPubKey as SignatureKey>::PrivateKey::generate(&mut rand::thread_rng());
        let new_signature = BLSPubKey::from_private(&private_key);
        let mut reg_ns_2 = reg_ns_1.clone();
        reg_ns_2.namespace_id = 2_u64.into();
        // Updating the registration with new signature generated
        reg_ns_2.signature = new_signature;

        // This should fail due to an invalid signature
        let err: SolverError = client
            .post::<RollupRegistration>("register_rollup")
            .body_json(&reg_ns_2)
            .unwrap()
            .send()
            .await
            .unwrap_err();

        // Ensure the error indicates an invalid signature
        match err {
            SolverError::InvalidSignature(signature)
                if reg_ns_2.signature.to_string() == signature => {}
            _ => panic!("err {err:?}"),
        }

        // Test the update rollup endpoint
        // We will use the existing rollup registration with namespace id = 1
        // and update it by setting the `active`` status to false
        let update_rolup = RollupUpdate {
            namespace_id: reg_ns_1.namespace_id,
            reserve_price: None,
            active: Some(false),
            signature_keys: None,
            text: None,
            signature,
        };

        // The result should contain the updated rollup registration data
        let result: RollupRegistration = client
            .post("update_rollup")
            .body_json(&update_rolup)
            .unwrap()
            .send()
            .await
            .unwrap();

        reg_ns_1.active = false;

        // Ensure the update result matches the modified registration data
        assert_eq!(reg_ns_1, result);

        // Test `rollup_registrations` endpoint to get all the registered rollups

        // The result should contain the updated rollup registration data
        let result: Vec<RollupRegistration> =
            client.get("rollup_registrations").send().await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], reg_ns_1);
    }

    #[async_std::test]
    async fn test_update_rollup_not_registered() {
        let mock_solver = MockSolver::init().await;
        let solver_api = mock_solver.solver_api();
        let client =
            surf_disco::Client::<SolverError, <SeqTypes as NodeType>::Base>::new(solver_api);

        let private_key =
            <BLSPubKey as SignatureKey>::PrivateKey::generate(&mut rand::thread_rng());
        let signature = BLSPubKey::from_private(&private_key);
        let update_rolup = RollupUpdate {
            namespace_id: 1_u64.into(),
            reserve_price: None,
            active: Some(false),
            signature_keys: None,
            text: None,
            signature,
        };

        let err: SolverError = client
            .post::<RollupUpdate>("update_rollup")
            .body_json(&update_rolup)
            .unwrap()
            .send()
            .await
            .unwrap_err();

        match err {
            SolverError::Database(_) => {}
            _ => panic!("err {err:?}"),
        }
    }

    #[async_std::test]
    async fn test_solver_api() {
        let mock_solver = MockSolver::init().await;

        let solver_api = mock_solver.solver_api();

        let client =
            surf_disco::Client::<SolverError, <SeqTypes as NodeType>::Base>::new(solver_api);

        let result: String = client.post("submit_bid").send().await.unwrap();
        assert_eq!(result, "Bid Submitted".to_string());
    }
}
