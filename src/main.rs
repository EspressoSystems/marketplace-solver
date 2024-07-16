use async_std::sync::RwLock;
use futures::FutureExt;
use serde::{Deserialize, Serialize};
use snafu::Snafu;
use std::{io, io::ErrorKind};
use tide_disco::{
    api::ApiError,
    method::{ReadState, WriteState},
    Api, App, StatusCode,
};
use vbs::version::{StaticVersion, StaticVersionType};

type StaticVer01 = StaticVersion<0, 1>;
pub const SOLVER_MAJOR_VERSION: u16 = 0;
pub const SOLVER_MINOR_VERSION: u16 = 1;
pub type SolverVersion = StaticVersion<SOLVER_MAJOR_VERSION, SOLVER_MINOR_VERSION>;
pub const SOLVER_VERSION: SolverVersion = StaticVersion {};

#[derive(Clone, Debug, Deserialize, Serialize, Snafu)]
enum SolverError {
    DefaultError { status: StatusCode, message: String },
}

impl tide_disco::Error for SolverError {
    fn catch_all(status: StatusCode, message: String) -> Self {
        Self::DefaultError { status, message }
    }

    fn status(&self) -> StatusCode {
        match self {
            Self::DefaultError { status, .. } => *status,
        }
    }
}

// TODO ED: Implement a shared solver state with the HotShot events received
#[derive(Default, Clone)]
struct SolverState {}

fn define_api<SolverState, SolverError, VERSION>(
) -> Result<Api<SolverState, SolverError, VERSION>, ApiError>
where
    VERSION: StaticVersionType + 'static,
    SolverState: 'static + Send + Sync + ReadState + WriteState,
    <SolverState as ReadState>::State: Send + Sync,
    SolverError: 'static,
{
    let api_toml = toml::from_str::<toml::Value>(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/api.toml"
    )))
    .expect("API file is not valid toml");

    let mut api = Api::new(api_toml)?;

    // TODO ED: We need to fill these in with the appropriate logic later
    api.post("submit_bid", |_req, _state| {
        async move { Ok("Bid Submitted") }.boxed()
    })?
    .get("auction_results", |_req, _state| {
        async move { Ok("Auction Results Gotten") }.boxed()
    })?
    .get("auction_results_permissioned", |_req, _state| {
        async move { Ok("Permissioned Auction Results Gotten") }.boxed()
    })?
    .post("register_rollup", |_req, _state| {
        async move { Ok("Rollup Registered") }.boxed()
    })?
    .post("update_rollup", |_req, _state| {
        async move { Ok("Rollup Updated") }.boxed()
    })?
    .get("rollup_registrations", |_req, _state| {
        async move { Ok("Rollup Registrations Gotten") }.boxed()
    })?;
    Ok(api)
}

#[async_std::main]
pub async fn main() {
    let mut app = App::<_, SolverError>::with_state(RwLock::new(SolverState::default()));
    app.with_version(env!("CARGO_PKG_VERSION").parse().unwrap());

    let mut api = define_api()
        .map_err(|_e| io::Error::new(ErrorKind::Other, "Failed to define api"))
        .unwrap();
    api.with_version(env!("CARGO_PKG_VERSION").parse().unwrap());

    app.register_module::<SolverError, SolverVersion>("hello", api)
        .unwrap();
    let _ = app.serve(format!("0.0.0.0:{}", 7777), StaticVer01::instance())
        .await;
}
