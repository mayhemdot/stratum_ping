//! Measuring the amount of time required to connect and successfully
//! pass authentication using the Stratum protocol
#![deny(missing_docs)]
#![warn(missing_debug_implementations, rust_2018_idioms, rustdoc::all)]

/// Error
pub mod error;
/// Request options
pub mod opts;
/// Stratum mining protocol types
pub mod query;
/// R|W new data to server by using stratum protocol
pub mod req;

use clap::Parser;
use opts::Opts;
use query::PingMultQuery;
use req::{OutgoingRequest, Result};

fn main() -> Result<()> {
    let opts = Opts::parse();
    OutgoingRequest::new(opts)?.ping_multiple()?;
    Ok(())
}
