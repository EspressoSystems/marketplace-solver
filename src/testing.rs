#![cfg(any(test, feature = "testing"))]
#![allow(dead_code)]
use std::sync::Arc;

use async_compatibility_layer::art::async_spawn;
use async_std::{sync::RwLock, task::JoinHandle};
use espresso_types::SeqTypes;
use hotshot_query_service::data_source::sql::testing::TmpDb;
use hotshot_types::traits::node_implementation::NodeType;
use portpicker::pick_unused_port;
use surf_disco::Client;
use tide_disco::{App, Url};
use vbs::version::StaticVersionType;

use crate::{
    database::{test::setup_mock_database, PostgresClient},
    define_api, handle_events,
    mock::run_mock_event_service,
    state::{GlobalState, SolverState, StakeTable},
    EventsServiceClient, SolverError,
};

pub struct MockSolver {
    events_api: Url,
    solver_api: Url,
    state: Arc<RwLock<GlobalState>>,
    database: PostgresClient,
    handles: Vec<JoinHandle<()>>,
    tmp_db: TmpDb,
}

impl MockSolver {
    fn solver_api(&self) -> Url {
        self.solver_api.clone()
    }

    fn state(&self) -> Arc<RwLock<GlobalState>> {
        self.state.clone()
    }
}

impl Drop for MockSolver {
    fn drop(&mut self) {
        println!("dropping handles");

        while let Some(handle) = self.handles.pop() {
            async_std::task::block_on(handle.cancel());
        }
    }
}

impl MockSolver {
    async fn init() -> Self {
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
            GlobalState::new(database.clone(), solver_state)
                .await
                .unwrap(),
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

#[async_std::test]
async fn test_solver_api() {
    let mock_solver = MockSolver::init().await;

    let solver_api = mock_solver.solver_api();

    let client = Client::<SolverError, <SeqTypes as NodeType>::Base>::new(solver_api);

    let result: String = client.post("submit_bid").send().await.unwrap();
    assert_eq!(result, "Bid Submitted".to_string());
}
