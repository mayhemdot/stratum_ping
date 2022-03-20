use super::error::{Error, ParseUrlError};
use super::opts::Opts;
use super::query::{Method, PingMultQuery, PingQuery, QRequest};

use miniserde::json;
use native_tls::TlsConnector;

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Instant;
use std::{net::ToSocketAddrs, time::Duration};

const SAMPLING_WIDTH_LIMIT: usize = 2000;

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

        let json_string: String = json::to_string(&req_buf);
        let mut json_buf = json_string.into_bytes();
        json_buf.push(10); // \n
        let mut buf = vec![0u8; 512];
        let start = Instant::now();

        conn.write_all(&json_buf)?;
        conn.read(&mut buf)?;

        Ok(start.elapsed())
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
            sampling_width,
            tls,
            proto,
            login,
            pass,
            timeout,
            ..
        } = &self.opts;

        if *sampling_width > SAMPLING_WIDTH_LIMIT {
            return Err(Error::InvalidNumberOfReplies);
        }

        let addr = self.address()?;

        println!(
            "{}\n{}",
            Status::Init,
            PingInitEntry {
                login: &login,
                pass: &pass,
                proto: &proto,
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

        for i in 0..*sampling_width {
            let elapsed = match self.ping() {
                Ok(t) => t,
                Err(e) => {
                    println!("[Error]: {:?}", e);
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

        print!(
            "{}\n{}",
            Status::Statistics,
            PingStateLine {
                title: "packets",
                col1: format!("transmitted={sampling_width}"),
                col2: format!("received={success}"),
                col3: format!("failure={} ", *sampling_width - success),
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
        write!(
            f,
            "{:<w$} \u{2502} {:>w$} \u{2502} {:>w$} \u{2502} {:>w$} \u{2502} {:<w$}\n",
            self.title,
            self.col1,
            self.col2,
            self.col3,
            self.col4,
            w = COL_W
        )?;

        Ok(())
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
            match self.proto.as_ref() {
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
