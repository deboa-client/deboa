//! Error types for the Deboa HTTP client.
//!
//! This module contains the error types used throughout the Deboa HTTP client.
//! The main error type is `DeboaError`, which can represent various kinds of errors
//! that might occur during HTTP requests and responses.
//!
//! # Error Types
//!
//! - `DeboaError`: The main error type that can represent any error that occurs in Deboa
//! - `RequestError`: Errors that occur while building or sending HTTP requests
//! - `ResponseError`: Errors that occur while processing HTTP responses
//! - `ConnectionError`: Errors that occur during network connections
//! - `ContentError`: Errors related to content serialization/deserialization
//! - `IoError`: I/O related errors
//!
//! # Examples
//!
//! ## Handling Errors
//!
//! ```compile_fail
//! use deboa::{Deboa, request::get};
//! use deboa::errors::DeboaError;
//!
//! #[tokio::main]
//! async fn main() -> TestResult<()> {
//!   let mut client = Deboa::default();
//!
//!   match get("https://example.com").and_then(|req| req.send_with(&mut client)) {
//!     Ok(response) => {
//!         // Handle successful response
//!     },
//!     Err(DeboaError::Connection(e)) => {
//!         // Handle connection errors
//!         eprintln!("Connection failed: {}", e);
//!     },
//!     Err(DeboaError::Request(e)) => {
//!         // Handle request errors
//!         eprintln!("Request failed: {}", e);
//!     },
//!     Err(e) => {
//!         // Handle other errors
//!         eprintln!("Error: {}", e);
//!     }
//!   }
//!   Ok(())
//! }
//! ``
use http::StatusCode;
use thiserror::Error;

/// The main error type for the Deboa HTTP client.
///
/// This enum represents all possible errors that can occur when making HTTP requests
/// with Deboa. It uses the `thiserror` crate to provide detailed error messages
/// and source information.
#[derive(Debug, Clone, Error, PartialEq)]
pub enum DeboaError {
    /// Invalid cookie header error
    #[error("Invalid cookie header: {message}")]
    Cookie {
        /// Error message
        message: String,
    },

    /// Invalid cookie header error
    #[error("Could not encode form: {message}")]
    Form {
        /// Error message
        message: String,
    },

    /// Invalid client certificate error (deprecated, use Identity instead)
    #[deprecated = "Use `Identity` instead"]
    #[error("Invalid client certificate: {message}")]
    ClientCert {
        /// Error message
        message: String,
    },

    /// Invalid certificate error
    #[error("Invalid certificate: {message}")]
    Certificate {
        /// Error message
        message: String,
    },

    /// Invalid identity error
    #[error("Invalid identity: {message}")]
    Identity {
        /// Error message
        message: String,
    },

    /// Invalid header error
    #[error("Invalid header: {message}")]
    Header {
        /// Error message
        message: String,
    },

    /// Connection error
    #[error("Connection error: {0}")]
    Connection(#[from] ConnectionError),

    /// DNS resolve error
    #[error("DNS resolve error: {0}")]
    Dns(#[from] DnsError),

    /// Request error
    #[error("Request error: {0}")]
    Request(#[from] RequestError),

    /// Response error
    #[error("Response error: {0}")]
    Response(#[from] ResponseError),

    /// Content error
    #[error("Content error: {0}")]
    Content(#[from] ContentError),

    /// Io error
    #[error("Io error: {0}")]
    Io(#[from] IoError),

    /// Unsupported protocol
    #[error("Unsupported protocol")]
    UnsupportedProtocol,
}

/// Request error
#[derive(Debug, Clone, Error, PartialEq)]
pub enum RequestError {
    /// Failed to parse request
    #[error("Failed to parse request: {message}")]
    Parse {
        /// Error message
        message: String,
    },

    /// Failed to send request
    #[error("Failed to send request: {message}")]
    Send {
        /// Error message
        message: String,
    },

    /// Failed to prepare request
    #[error("Failed to prepare request: {message}")]
    Prepare {
        /// Error message
        message: String,
    },

    /// Failed to parse url
    #[error("Failed to parse url: {message}")]
    UrlParse {
        /// Error message
        message: String,
    },

    /// Failed to parse method
    #[error("Failed to parse method: {message}")]
    MethodParse {
        /// Error message
        message: String,
    },

    /// Request timeout error
    #[error("Request timeout: {message}")]
    Timeout {
        /// Error message
        message: String,
    },
}

/// Connection error
#[derive(Debug, Clone, Error, PartialEq)]
pub enum ConnectionError {
    /// Tcp connection error
    #[error("Tcp error: {message}")]
    Tcp {
        /// Error message
        message: String,
    },

    /// Tls connection error
    #[error("Tls error: {message}")]
    Tls {
        /// Error message
        message: String,
    },

    /// Udp connection error
    #[error("Udp error: {message}")]
    Udp {
        /// Error message
        message: String,
    },

    /// Connection handshake error
    #[error("Handshake error: {message}")]
    Handshake {
        /// Error message
        message: String,
    },

    /// Connection upgrade error
    #[error("Connection upgrade error: {message}")]
    Upgrade {
        /// Error message
        message: String,
    },

    /// Connection timeout error
    #[error("Connection timeout: {message}")]
    Timeout {
        /// Error message
        message: String,
    },

    /// Unsupported scheme error
    #[error("Unsupported scheme: {message}")]
    UnsupportedScheme {
        /// Error message
        message: String,
    },
}

/// Content error
#[derive(Debug, Clone, Error, PartialEq)]
pub enum ContentError {
    /// Failed to serialize data
    #[error("Failed to serialize data: {message}")]
    Serialization {
        /// Error message
        message: String,
    },

    /// Failed to deserialize data
    #[error("Failed to deserialize data: {message}")]
    Deserialization {
        /// Error message
        message: String,
    },
}

/// WebSocket errors
#[derive(Debug, Clone, Error, PartialEq)]
pub enum WebSocketError {
    /// Failed to send message
    #[error("Failed to send message: {message}")]
    SendMessage {
        /// The error message
        message: String,
    },

    /// Failed to receive message
    #[error("Failed to receive message: {message}")]
    ReceiveMessage {
        /// The error message
        message: String,
    },
}

/// Connection error
#[derive(Debug, Clone, Error, PartialEq)]
pub enum DnsError {
    /// Failed to resolve DNS
    #[error("Failed to resolve DNS: {message}")]
    Resolve {
        /// Host
        host: String,
        /// Error message
        message: String,
    },

    /// Failed to initialize resolver
    #[error("Failed to initialize resolver: {message}")]
    Resolver {
        /// Error message
        message: String,
    },
}

/// Io error
#[derive(Debug, Clone, Error, PartialEq)]
pub enum IoError {
    /// Failed to read content
    #[error("Failed to read content: {message}")]
    Content {
        /// Error message
        message: String,
    },

    /// Failed to write file
    #[error("Failed to write file: {message}")]
    File {
        /// Error message
        message: String,
    },

    /// Failed to compress data
    #[error("Failed to compress data: {message}")]
    Compress {
        /// Error message
        message: String,
    },

    /// Failed to decompress data
    #[error("Failed to decompress data: {message}")]
    Decompress {
        /// Error message
        message: String,
    },

    /// Failed to write to stdout
    #[error("Failed to write to stdout: {message}")]
    Stdout {
        /// Error message
        message: String,
    },

    /// Failed to write to stderr
    #[error("Failed to write to stderr: {message}")]
    Stderr {
        /// Error message
        message: String,
    },

    /// Failed to read from stdin
    #[error("Failed to read from stdin: {message}")]
    Stdin {
        /// Error message
        message: String,
    },
}

/// Response error
#[derive(Debug, Clone, Error, PartialEq)]
pub enum ResponseError {
    /// Failed to receive response
    #[error("Failed to receive response: {status_code}: {message}")]
    Receive {
        /// Status code
        status_code: StatusCode,
        /// Error message
        message: String,
    },

    /// Failed to process response
    #[error("Failed to process response: {message}")]
    Process {
        /// Error message
        message: String,
    },
}
