use async_std::sync::RwLock;
use marketplace_solver::{define_api, state::SolverState, SolverError};
use std::{
    io::{self, ErrorKind},
    sync::Arc,
};
use tide_disco::App;
use vbs::version::{StaticVersion, StaticVersionType};

type StaticVer01 = StaticVersion<0, 1>;
pub const SOLVER_MAJOR_VERSION: u16 = 0;
pub const SOLVER_MINOR_VERSION: u16 = 1;
pub type SolverVersion = StaticVersion<SOLVER_MAJOR_VERSION, SOLVER_MINOR_VERSION>;
pub const SOLVER_VERSION: SolverVersion = StaticVersion {};

// >>>> main() will be removed when we move binary to sequencer
#[async_std::main]
pub async fn main() {
    let mut app = App::<_, SolverError>::with_state(Arc::new(RwLock::new(SolverState::default())));
    app.with_version(env!("CARGO_PKG_VERSION").parse().unwrap());

    let mut api = define_api()
        .map_err(|_e| io::Error::new(ErrorKind::Other, "Failed to define api"))
        .unwrap();
    api.with_version(env!("CARGO_PKG_VERSION").parse().unwrap());

    app.register_module::<SolverError, SolverVersion>("hello", api)
        .unwrap();
    let _ = app
        .serve(format!("0.0.0.0:{}", 7777), StaticVer01::instance())
        .await;
}
