//! Measuring the amount of time required to connect and successfully
//! pass authentication using the Stratum protocol
#![deny(missing_docs)]
#![warn(missing_debug_implementations, rust_2018_idioms, rustdoc::all)]

/// Error
pub mod error;
/// Request options
pub mod opts;
/// The Stratum mining protocol types
pub mod query;
/// R/W data to server
pub mod req;

use clap::Parser;
use opts::Opts;
use query::PingMultQuery;
use req::OutgoingRequest;

fn main() -> req::Result<()> {
    init_log();

    let opts = Opts::parse();
    OutgoingRequest::new(opts)?.ping_multiple()?;
    Ok(())
}

fn init_log() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "error")
    }
     env_logger::init();
}
