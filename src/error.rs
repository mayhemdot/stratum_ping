use std::{borrow::Cow, fmt::Display, net::TcpStream};

/// An error occurred while attempting to establish a session for a new `Client`.
#[derive(Debug)]
pub enum Error {
    /// An error returned when The given URL is invalid.
    URL(ParseUrlError),
    /// An error which can be returned when total request limit exceeded.
    InvalidCountReplies,
    /// An error which can be returned when tnvalid stratum type specified.
    InvalidStratumType,
    /// A standard I/O error occurred.
    Io(std::io::Error),
    /// A standard `native_tls` error occurred.
    TLS(native_tls::Error),
    /// An error returned from `native_tls`.
    HandshakeTLSOverTCP(native_tls::HandshakeError<TcpStream>),
    /// A standard ParseIntError error occurred.
    ParseInt(std::num::ParseIntError),
}

/// Errors that can occur during parsing.
#[allow(missing_docs)]
#[derive(Debug)]
pub enum ParseUrlError {
    EmptyHost,
    EmptyPort,
    InvalidHost(Cow<'static, str>),
    InvalidPort(Cow<'static, str>),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(e: std::num::ParseIntError) -> Self {
        Error::ParseInt(e)
    }
}

impl From<ParseUrlError> for Error {
    fn from(e: ParseUrlError) -> Self {
        Error::URL(e)
    }
}

impl From<native_tls::Error> for Error {
    fn from(e: native_tls::Error) -> Self {
        Error::TLS(e)
    }
}

impl From<native_tls::HandshakeError<TcpStream>> for Error {
    fn from(e: native_tls::HandshakeError<TcpStream>) -> Self {
        Error::HandshakeTLSOverTCP(e)
    }
}
impl Display for ParseUrlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ParseUrlError::*;
        match self {
            EmptyHost => write!(f, "empty host"),
            EmptyPort => write!(f, "empty port number"),
            InvalidPort(s) => write!(f, "invalid port number: {}", s),
            InvalidHost(s) => write!(f, "invalid host name: {}", s),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;
        match self {
            InvalidStratumType => write!(f, "Invalid stratum protocol version specified"),
            InvalidCountReplies => write!(f, "Invalid count replies specified"),
            URL(s) => write!(f, "{}", s),
            TLS(e) => write!(f, "{}", e),
            Io(e) => write!(f, "{}", e),
            ParseInt(e) => write!(f, "{}", e),
            HandshakeTLSOverTCP(e) => write!(f, "{}", e),
        }
    }
}
