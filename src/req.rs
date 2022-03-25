use super::error::{Error, ParseUrlError};
use super::opts::Opts;
use super::query::{Method, PingMultQuery, PingQuery, QRequest};

use miniserde::json;
use native_tls::TlsConnector;

use log::{debug, error};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Instant;
use std::{net::ToSocketAddrs, time::Duration};

/// A samples width
const SAMPLES_LIMIT: usize = 2000;

/// A specialized [Result] type for request operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Request periodically new data to server by using stratum protocol.
pub(crate) struct OutgoingRequest {
    opts: Opts,
    host: String,
}

impl OutgoingRequest {
    /// Steps:
    /// - Parsing hostname from string
    /// - Looking up an address IP end point
    pub(crate) fn new(opts: Opts) -> Result<Self> {
        use ParseUrlError::*;
        Ok(Self {
            host: match opts.server.split_terminator(':').next() {
                Some(host) => host.into(),
                None => return Err(EmptyHost.into()),
            },
            opts,
        })
    }

    pub(crate) fn address(&self) -> Result<SocketAddr> {
        Ok(self.opts.server.to_socket_addrs()?.next().unwrap())
    }
}

pub(crate) trait ReadWrite
where
    Self: Write + Read,
{
}

impl<T> ReadWrite for T where T: Write + Read {}

impl PingQuery<Duration> for OutgoingRequest {
    type Output = Duration;
    type Err = Error;

    fn ping(&self) -> std::result::Result<Self::Output, Self::Err> {
        let Opts {
            proto,
            login,
            pass,
            tls,
            timeout,
            ..
        } = &self.opts;

        let stream = TcpStream::connect(self.address()?)?;
        let timeout = Duration::from_secs(*timeout);

        stream.set_write_timeout(Some(timeout))?;
        stream.set_read_timeout(Some(timeout))?;

        let mut conn: Box<dyn ReadWrite> = match tls {
            true => {
                let mut tls = TlsConnector::builder();
                tls.danger_accept_invalid_certs(true);
                tls.danger_accept_invalid_hostnames(true);
                Box::new(tls.build()?.connect(&self.host, stream)?)
            }
            false => Box::new(stream),
        };

        let req_buf = match proto.as_ref() {
            "stratum1" => {
                QRequest::new(1, Method::EthSubmitLogin, vec![login.clone(), pass.clone()])
            }
            "stratum2" => QRequest::new(
                1,
                Method::MiningSubscribe,
                vec![
                    "stratum-ping/1.0.0".to_string(),
                    "EthereumStratum/1.0.0".to_string(),
                ],
            ),
            _ => return Err(Error::InvalidStratumType),
        };

        let mut send_msg: String = json::to_string(&req_buf);
        send_msg.push('\n');

        let mut buf = vec![0u8; 512];
        debug!("{}", &send_msg);

        let start = Instant::now();
        conn.write_all(send_msg.as_bytes())?;

        let num = conn.read(&mut buf)?;

        let elapsed = start.elapsed();

        let recv_msg = String::from_utf8_lossy(&buf[..num]);
        debug!("{}", &recv_msg);

        Ok(elapsed)
    }
}

impl PingMultQuery<()> for OutgoingRequest
where
    Self: PingQuery<Duration>,
{
    type Err = Error;
    type Output = ();

    fn ping_multiple(&self) -> std::result::Result<Self::Output, Self::Err> {
        let Opts {
            samples,
            tls,
            proto,
            login,
            pass,
            timeout,
            ..
        } = &self.opts;

        if *samples > SAMPLES_LIMIT {
            return Err(Error::InvalidNumberOfReplies);
        }

        let addr = self.address()?;

        println!(
            "{}\n{}",
            Status::Init,
            PingInitEntry {
                login,
                pass,
                proto,
                host: &self.host,
                tls: *tls,
                timeout: *timeout,
                addr,
            }
        );

        let (mut min, mut max, mut avg) =
            (Duration::from_secs(60 * 60), Duration::ZERO, Duration::ZERO);

        let mut success: usize = 0;
        let start = Instant::now();

        println!("{}", Status::SendRecv);

        for i in 0..*samples {
            let elapsed = match self.ping() {
                Ok(t) => t,
                Err(e) => {
                    error!("[Error]: {:?}", e);
                    continue;
                }
            };
            println!(
                "{:?} ({:?}): seq={}, time={:?}",
                self.host, addr, i, elapsed
            );
            if elapsed > max {
                max = elapsed;
            }
            if elapsed < min {
                min = elapsed;
            }

            avg += elapsed;
            success += 1;

            std::thread::sleep(Duration::from_secs(1));
        }

        println!(
            "{}\n{}",
            Status::Statistics,
            PingStateLine {
                title: "packets",
                col1: format!("transmitted={samples}"),
                col2: format!("received={success}"),
                col3: format!("failure={} ", *samples - success),
                col4: String::new(),
            }
        );

        if success > 0 {
            println!(
                "{}",
                PingStateLine {
                    title: "time",
                    col1: format!("min={min:.2?}"),
                    col2: format!("avg={:.2?}", avg / success as u32),
                    col3: format!("max={max:.2?}"),
                    col4: format!("total={:.2?}", start.elapsed()),
                }
            );
        }
        Ok(())
    }
}

/// Column width used in formatting entries
const COL_W: usize = 15;
const COL_H: usize = 80;

pub(crate) enum Status {
    Init,
    SendRecv,
    Statistics,
}

pub(crate) struct PingInitEntry<'a> {
    login: &'a str,
    pass: &'a str,
    proto: &'a str,
    host: &'a str,
    addr: SocketAddr,
    tls: bool,
    timeout: u64,
}

pub(crate) struct PingStateLine<'a, T>
where
    T: std::fmt::Display,
{
    title: &'a str,
    col1: T,
    col2: T,
    col3: T,
    col4: T,
}

impl<'a, T> std::fmt::Display for PingStateLine<'a, T>
where
    T: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(write!(
            f,
            "{:<w$} \u{2502} {:>w$} \u{2502} {:>w$} \u{2502} {:>w$} \u{2502} {:<w$}",
            self.title,
            self.col1,
            self.col2,
            self.col3,
            self.col4,
            w = COL_W
        )?)
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let line = format!("{:\u{2500}<COL_H$}", "");
        match self {
            Status::Init => write!(f, "{}\n[Initialization]\n{}", line, line),
            Status::SendRecv => write!(f, "{}\n[Send/Recv]\n{}", line, line),
            Status::Statistics => write!(f, "{}\n[Statistics]\n{}", line, line),
        }
    }
}
impl std::fmt::Display for PingInitEntry<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:w$} \u{2502} {}\n{:w$} \u{2502} {}\n{:w$} \u{2502} {}\n{:w$} \u{2502} {} sec\n{:w$} \u{2502} {}{}",
            "host",
            self.host,
            "addr",
            self.addr,
            "protocol",
            self.proto,
            "timeout",
            self.timeout,
            "tls",
            self.tls,
            match self.proto {
                "stratum1" => format!(
                    "\n{:COL_W$} \u{2502} {}:{}",
                    "credentials", self.login, self.pass
                ),
                _ => String::new(),
            },
            w = COL_W,
        )
    }
}
