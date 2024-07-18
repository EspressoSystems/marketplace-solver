use std::io::{self, ErrorKind};

use async_std::sync::RwLock;
use clap::Parser;
use espresso_types::SeqTypes;
use hotshot_events_service::events_source::StartupInfo;
use marketplace_solver::{
    define_api,
    state::{GlobalState, SolverState, StakeTable},
    Options, SolverError,
};
use surf_disco::Client;
use tide_disco::App;
use vbs::version::{StaticVersion, StaticVersionType};

type StaticVer01 = StaticVersion<0, 1>;
pub const SOLVER_MAJOR_VERSION: u16 = 0;
pub const SOLVER_MINOR_VERSION: u16 = 1;
pub type SolverVersion = StaticVersion<SOLVER_MAJOR_VERSION, SOLVER_MINOR_VERSION>;
pub const SOLVER_VERSION: SolverVersion = StaticVersion {};

// >>>> main() will be removed when we move binary to sequencer
//  this demonstrate the intended flow
#[async_std::main]
pub async fn main() {
    let options = Options::parse();
    let database_options = options.database_options;

    let events_api = options.events_url;

    let client = Client::<hotshot_events_service::events::Error, StaticVer01>::new(events_api);
    client.connect(None).await;

    let startup_info: StartupInfo<SeqTypes> = client
        .get("startup_info")
        .send()
        .await
        .expect("failed to get startup_info");

    let solver_state = SolverState {
        stake_table: StakeTable {
            known_nodes_with_stake: startup_info.known_node_with_stake,
        },
    };

    let state = GlobalState::new(database_options, solver_state)
        .await
        .unwrap();

    let mut app = App::<_, SolverError>::with_state(RwLock::new(state));
    app.with_version(env!("CARGO_PKG_VERSION").parse().unwrap());

    let mut api = define_api(Default::default())
        .map_err(|_e| io::Error::new(ErrorKind::Other, "Failed to define api"))
        .unwrap();
    api.with_version(env!("CARGO_PKG_VERSION").parse().unwrap());

    app.register_module::<SolverError, SolverVersion>("hello", api)
        .unwrap();
    let _ = app
        .serve(format!("0.0.0.0:{}", 7777), StaticVer01::instance())
        .await;
}
