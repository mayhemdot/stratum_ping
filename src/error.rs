use std::{borrow::Cow, fmt::Display, net::TcpStream};

/// An error occurred while attempting to establish a session for a new `Client`.
#[derive(Debug)]
pub enum Error {
    /// An error returned when The given URL is invalid.
    URL(ParseUrlError),
    /// An error which can be returned when total request limit exceeded.
    InvalidNumberOfReplies,
    /// An error which can be returned when tnvalid stratum type specified.
    InvalidStratumType,
    /// An `io::Error` that occurred while trying to read or write to a network stream
    Io(std::io::Error),
    /// A standard `native_tls` error occurred.
    TLS(native_tls::Error),
    /// Error occurs when the server are unable to establish a secure connection.
    HandshakeTLSOverTCP(Box<native_tls::HandshakeError<TcpStream>>),
    /// An error which can be returned when parsing an integer.
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
        Error::HandshakeTLSOverTCP(Box::new(e))
    }
}
impl Display for ParseUrlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ParseUrlError::*;
        match self {
            EmptyHost => write!(f, "Empty host name"),
            EmptyPort => write!(f, "Empty port number"),
            InvalidPort(s) => write!(f, "Invalid port number: {}", s),
            InvalidHost(s) => write!(f, "Invalid host name: {}", s),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;
        match self {
            InvalidStratumType => write!(f, "Invalid stratum protocol version specified"),
            InvalidNumberOfReplies => write!(f, "Invalid the number of requests per timeframe"),
            URL(s) => write!(f, "{}", s),
            TLS(e) => write!(f, "{}", e),
            Io(e) => write!(f, "{}", e),
            ParseInt(e) => write!(f, "{}", e),
            HandshakeTLSOverTCP(e) => write!(f, "{}", e),
        }
    }
}
