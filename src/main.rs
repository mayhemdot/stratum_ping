//! Measuring the amount of time required to connect and successfully
//! pass authentication using the Stratum protocol
#![deny(missing_docs)]
#![warn(missing_debug_implementations, rust_2018_idioms, rustdoc::all)]

/// Request options
pub mod args;
/// Error
pub mod error;
/// Stratum mining protocol types.
pub mod query;
/// Request periodically new data to server by using stratum protocol.
pub mod req;

use clap::Parser;
use query::PingMultQuery;
use req::Result;

fn main() -> Result<()> {
    let opts = args::RequestOpts::parse();
    req::OutgoingRequest::new(opts)?.ping_multiple()?;
    Ok(())
}
