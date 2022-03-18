use super::error::{Error, ParseUrlError};
use super::opts::Opts;
use super::query::{Method, PingMultQuery, PingQuery, QRequest};

use miniserde::json;
use native_tls::TlsConnector;

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Instant;
use std::{net::ToSocketAddrs, time::Duration};

const CONNECTION_ATTEMPTS_LIMIT: usize = 2000;

/// A specialized [Result] type for request operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Request periodically new data to server by using stratum protocol.
pub(crate) struct OutgoingRequest {
    opts: Opts,
    host: String,
    addr: Option<SocketAddr>,
}

impl OutgoingRequest {
    /// Steps:
    /// - Parsing hostname from string
    /// - Looking up an address IP end point
    pub(crate) fn new(opts: Opts) -> Result<Self> {
        use ParseUrlError::*;

        let mut server = opts.server.split_terminator(':');
        Ok(Self {
            host: match server.next() {
                Some(host) => host.into(),
                None => return Err(EmptyHost.into()),
            },
            addr: opts.server.to_socket_addrs()?.next(),
            opts,
        })
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
            ..
        } = &self.opts;

        let stream = TcpStream::connect(self.addr.unwrap())?;
        let timeout = Duration::from_secs(10);

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
            attempts,
            tls,
            proto,
            login,
            pass,
            ..
        } = &self.opts;

        if *attempts > CONNECTION_ATTEMPTS_LIMIT {
            return Err(Error::InvalidNumberOfReplies);
        }

        println!(
            "\n[PING] {:?} {:?} ({:?})\ntls: {}{}\n",
            proto,
            self.host,
            self.addr.unwrap(),
            tls,
            match proto.as_ref() {
                "stratum1" => format!(", credentials: {login}:{pass}"),
                _ => String::new(),
            }
        );

        let (mut min, mut max, mut avg) =
            (Duration::from_secs(60 * 60), Duration::ZERO, Duration::ZERO);

        let mut success: usize = 0;
        let start = Instant::now();

        for i in 0..*attempts {
            let elapsed = self.ping()?;
            println!(
                "{:?} ({:?}): seq={}, time={:?}",
                self.host,
                self.addr.unwrap(),
                i,
                elapsed
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
        println!("\n[PING statistics]");
        let loss = 100 - (success as f64 / *attempts as f64 * 100_f64) as usize;
        println!(
            "{:<7} | {:>13} | {:>12} | {:>12} |",
            "packets",
            format!("{attempts} transmitted"),
            format!("{success} received"),
            format!(" {loss}% loss")
        );

        if success > 0 {
            let avg = avg / success as u32;
            println!(
                "{:<7} | {:>13} | {:>12} | {:>12} | {:<12}\n",
                "time",
                format!("min={min:.2?}"),
                format!("avg={avg:.2?}"),
                format!("max={max:.2?}"),
                format!("elapsed={:.2?}", start.elapsed()),
            );
        }
        Ok(())
    }
}
