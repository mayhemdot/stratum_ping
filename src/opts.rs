use clap::Parser;

#[allow(missing_docs)]
#[derive(Parser, Debug)]
pub struct Opts {
    /// Pool address <HOST:PORT>.
    #[clap(long)]
    pub(crate) server: String,
    /// Some pools require appending of user name and/or worker.
    #[clap(short = 'u', long, default_value = "X")]
    pub(crate) login: String,
    /// Most pools don't require password.
    #[clap(short = 'p', long, default_value = "x")]
    pub(crate) pass: String,
    /// The number of request that determine the average response time.
    #[clap(short = 's', long, default_value_t = 5)]
    pub(crate) samples: usize,
    /// Selects the kind of stratum protocol
    #[clap(long, default_value = "stratum2")]
    pub(crate) proto: String,
    #[clap(long, parse(try_from_str), default_value_t = false)]
    pub(crate) tls: bool,
    /// After this time (seconds), the client can close this connection.
    #[clap(long, default_value_t = 5)]
    pub(crate) timeout: u64,
}
