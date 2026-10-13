//! Native security boundary; no secret export API.
pub mod bitcoin;
pub mod capabilities;
pub mod entropy;
pub mod error;
pub mod model;
#[cfg(feature = "network")]
pub mod network;
pub mod protocol;
pub mod session;
pub mod storage;
pub mod wallet;
pub use error::{Error, Result};
