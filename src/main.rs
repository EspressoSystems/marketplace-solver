use std::sync::Arc;

use async_compatibility_layer::art::async_spawn;
use async_std::sync::RwLock;
use clap::Parser;
use marketplace_solver::{
    define_api, handle_events,
    state::{GlobalState, SolverState, StakeTable},
    EventsServiceClient, Options, SolverError,
};
use tide_disco::App;
use vbs::version::{StaticVersion, StaticVersionType};

type StaticVer01 = StaticVersion<0, 1>;
pub const SOLVER_MAJOR_VERSION: u16 = 0;
pub const SOLVER_MINOR_VERSION: u16 = 1;
pub type SolverVersion = StaticVersion<SOLVER_MAJOR_VERSION, SOLVER_MINOR_VERSION>;
pub const SOLVER_VERSION: SolverVersion = StaticVersion {};

// >>>> main() will be removed when we move binary to sequencer
//  this demonstrates the intended flow
#[async_std::main]
pub async fn main() {
    let options = Options::parse();
    let database_options = options.database_options;

    let events_api_url = options.events_url;

    let client = EventsServiceClient::new(events_api_url).await;
    let startup_info = client.get_startup_info().await.unwrap();
    let stream = client.get_event_stream().await.unwrap();

    let solver_state = SolverState {
        stake_table: StakeTable {
            known_nodes_with_stake: startup_info.known_node_with_stake,
        },
        bid_txs: Default::default(),
    };

    let db = database_options
        .connect()
        .await
        .expect("failed to create database");
    let state = Arc::new(RwLock::new(
        GlobalState::new(db, solver_state).await.unwrap(),
    ));

    let _handle = async_spawn(handle_events(stream, state.clone()));

    let mut app = App::<_, SolverError>::with_state(state);
    app.with_version(env!("CARGO_PKG_VERSION").parse().unwrap());

    let mut api = define_api(Default::default()).unwrap();
    api.with_version(env!("CARGO_PKG_VERSION").parse().unwrap());

    app.register_module::<SolverError, SolverVersion>("hello", api)
        .unwrap();
    let _ = app
        .serve(format!("http://0.0.0.0:{}", 7777), StaticVer01::instance())
        .await;
}
