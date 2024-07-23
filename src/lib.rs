mod api;
pub mod database;
mod events;
mod options;
pub mod state;
mod testing;
pub mod types;

pub use api::*;
pub use events::*;
pub use options::*;

type SolverResult<T> = Result<T, SolverError>;
