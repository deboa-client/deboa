//! DNS module for resolving hostnames to IP addresses.
//!
//! This module provides functionality for resolving hostnames to IP addresses.
use crate::Result;
use std::{future::Future, net::IpAddr};

/// DNS resolver trait for resolving hostnames to IP addresses.
///
/// The returned future is an associated type rather than a boxed
/// `dyn Future + Send`, which is what it used to be. Two reasons:
///
/// - **`Send` was more than a resolver can promise on every runtime.**
///   `getaddrinfo` blocks, so a resolver hands it to a thread pool — and where
///   that pool's handle is itself `!Send`, as glommio's is, the boxed `Send`
///   future was unimplementable. compio's `spawn_blocking` returns a `Send`
///   future and satisfied it; glommio's does not, and had to pull in a second
///   thread pool to work around a bound it could not meet.
/// - It removes an allocation per lookup, which matters less but is free here.
///
/// `impl Future` in trait position matches the style the connection traits in
/// [`crate::conn`] already use.
pub trait DnsResolver: Send + 'static {
    /// Resolves a hostname to a list of IP addresses.
    fn resolve(&self, host: String, port: u16) -> impl Future<Output = Result<Vec<IpAddr>>>;
}
