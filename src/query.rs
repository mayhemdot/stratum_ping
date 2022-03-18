use miniserde::{Deserialize, Serialize};

/// Client to server
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct QRequest<T> {
    jsonrpc: String,
    id: usize,
    method: String,
    params: Vec<T>,
}

impl<T> QRequest<T> {
    pub fn new(id: usize, method: Method, params: Vec<T>) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            method: format!("{}", method),
            params,
        }
    }
}

/// Stratum mining protocol methods
/// https://en.bitcoin.it/wiki/Stratum_mining_protocol
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(crate) enum Method {
    /// V2:Client
    MiningSubscribe,
    /// V1:Client
    EthSubmitLogin,
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Method::MiningSubscribe => write!(f, "mining.subscribe"),
            Method::EthSubmitLogin => write!(f, "eth_submitLogin"),
        }
    }
}

impl std::str::FromStr for Method {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "mining.subscribe" => Method::MiningSubscribe,
            "eth_submitlogin" => Method::EthSubmitLogin,
            _ => return Err(format!("unknown method: {}", s)),
        })
    }
}

#[allow(missing_docs)]
pub trait PingQuery<R> {
    type Err;
    type Output;
    fn ping(&self) -> std::result::Result<Self::Output, Self::Err>;
}

#[allow(missing_docs)]
pub trait PingMultQuery<R> {
    type Err;
    type Output;
    fn ping_multiple(&self) -> std::result::Result<Self::Output, Self::Err>;
}
