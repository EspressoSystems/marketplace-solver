use futures::FutureExt;
use serde::{Deserialize, Serialize};
use snafu::Snafu;
use tide_disco::{
    api::ApiError,
    method::{ReadState, WriteState},
    Api, StatusCode,
};
use vbs::version::StaticVersionType;

use crate::state::{SolverDataSource, UpdateSolverState};

#[derive(Clone, Debug, Deserialize, Serialize, Snafu)]
pub enum SolverError {
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

pub fn define_api<State, SolverError, VERSION>(
) -> Result<Api<State, SolverError, VERSION>, ApiError>
where
    VERSION: StaticVersionType + 'static,
    State: 'static + Send + Sync + ReadState + WriteState + UpdateSolverState,
    <State as ReadState>::State: Send + Sync + SolverDataSource,
    SolverError: 'static,
{
    let api_toml = toml::from_str::<toml::Value>(include_str!("../api/solver.toml"))
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
