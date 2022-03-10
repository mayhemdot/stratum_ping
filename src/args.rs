use clap::Parser;
use std::net::SocketAddr;

#[allow(missing_docs)]
#[derive(Parser, Debug)]
pub struct RequestOpts {
    /// Stratum server host:port
    #[clap(long)]
    pub(crate) server: String,
    #[clap(short = 'u', long, default_value = "X")]
    pub(crate) login: String,
    #[clap(short = 'p', long, default_value = "x")]
    pub(crate) pass: String,
    /// Stop after <COUNT> replies
    #[clap(short, long, default_value = "5")]
    pub(crate) count: isize,
    /// Use ipv6 or not
    #[clap(long)]
    pub(crate) ipv6: bool,
    #[clap(long)]
    pub(crate) host: Option<String>,
    #[clap(long)]
    pub(crate) port: Option<u16>,
    #[clap(long)]
    pub(crate) addr: Option<SocketAddr>,
    /// Stratum type: stratum1, stratum2
    #[clap(long, default_value = "stratum2")]
    pub(crate) proto: String,
    #[clap(long)]
    pub(crate) tls: Option<bool>,
    #[clap(long, default_value = "error")]
    pub(crate) log_level: String,
}
