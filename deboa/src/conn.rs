//! Connection module
//!
//! This module provides functionality for managing HTTP connections.
use crate::{
    cert::{Certificate, Identity},
    dns::DnsResolver,
    response::DeboaResponse,
    Result,
};
use http::{Request, Version};
use hyper_body_utils::HttpBody;
use std::time::Duration;
use std::{future::Future, net::IpAddr};

/// Builder for connection configuration.
pub struct ConnectionConfigBuilder<'a, I, C> {
    scheme: &'a str,
    host: &'a str,
    port: u16,
    protocol_version: Version,
    connection_timeout: Duration,
    identity: Option<&'a I>,
    certificate: Option<&'a C>,
    skip_cert_verification: bool,
    client_bind_addr: IpAddr,
    prior_knowledge: bool,
}

impl<'a, I, C> ConnectionConfigBuilder<'a, I, C>
where
    I: Identity,
    C: Certificate,
{
    /// Create a new connection configuration builder.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            scheme: "http",
            host: "",
            port: 80,
            protocol_version: Version::HTTP_2,
            connection_timeout: Duration::from_secs(30),
            identity: None,
            certificate: None,
            skip_cert_verification: false,
            client_bind_addr: "0.0.0.0"
                .parse()
                .unwrap(),
            prior_knowledge: false,
        }
    }

    /// Set the scheme for the connection.
    pub fn scheme(mut self, scheme: &'a str) -> Self {
        self.scheme = scheme;
        self
    }

    /// Set the host for the connection.
    pub fn host(mut self, host: &'a str) -> Self {
        self.host = host;
        self
    }

    /// Set the port for the connection.
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Set the protocol for the connection.
    pub fn protocol_version(mut self, protocol_version: Version) -> Self {
        self.protocol_version = protocol_version;
        self
    }

    /// Set the connection timeout for the connection.
    pub fn connection_timeout(mut self, connection_timeout: Duration) -> Self {
        self.connection_timeout = connection_timeout;
        self
    }

    /// Set the identity for the connection.
    pub fn identity(mut self, identity: Option<&'a I>) -> Self {
        self.identity = identity;
        self
    }

    /// Set the certificate for the connection.
    pub fn certificate(mut self, certificate: Option<&'a C>) -> Self {
        self.certificate = certificate;
        self
    }

    /// Set whether to skip certificate verification.
    pub fn skip_cert_verification(mut self, skip_cert_verification: bool) -> Self {
        self.skip_cert_verification = skip_cert_verification;
        self
    }

    /// Set the client bind address for the connection.
    pub fn client_bind_addr(mut self, client_bind_addr: IpAddr) -> Self {
        self.client_bind_addr = client_bind_addr;
        self
    }

    /// Set whether to prior knowdlege of protocol support.
    pub fn prior_knowledge(mut self, prior_knowledge: bool) -> Self {
        self.prior_knowledge = prior_knowledge;
        self
    }

    /// Build the connection configuration.
    pub fn build(self) -> ConnectionConfig<'a, I, C> {
        ConnectionConfig {
            scheme: self.scheme,
            host: self.host,
            port: self.port,
            protocol_version: self.protocol_version,
            connection_timeout: self.connection_timeout,
            identity: self.identity,
            certificate: self.certificate,
            skip_cert_verification: self.skip_cert_verification,
            client_bind_addr: self.client_bind_addr,
            prior_knowledge: self.prior_knowledge,
        }
    }
}

/// Connection configuration.
pub struct ConnectionConfig<'a, I, C> {
    scheme: &'a str,
    host: &'a str,
    port: u16,
    protocol_version: Version,
    connection_timeout: Duration,
    identity: Option<&'a I>,
    certificate: Option<&'a C>,
    skip_cert_verification: bool,
    client_bind_addr: IpAddr,
    prior_knowledge: bool,
}

impl<'a, I, C> ConnectionConfig<'a, I, C>
where
    I: Identity,
    C: Certificate,
{
    /// Create a new connection configuration builder.
    pub fn builder() -> ConnectionConfigBuilder<'a, I, C> {
        ConnectionConfigBuilder::new()
    }

    /// Get the scheme for the connection.
    pub fn scheme(&self) -> &str {
        self.scheme
    }

    /// Get the host for the connection.
    pub fn host(&self) -> &str {
        self.host
    }

    /// Get the port for the connection.
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Get the protocol for the connection.
    pub fn protocol_version(&self) -> &Version {
        &self.protocol_version
    }

    /// Get the connection timeout for the connection.
    pub fn connection_timeout(&self) -> Duration {
        self.connection_timeout
    }

    /// Get the identity for the connection.
    pub fn identity(&self) -> Option<&I> {
        self.identity
    }

    /// Get the certificate for the connection.
    pub fn certificate(&self) -> Option<&C> {
        self.certificate
    }

    /// Get whether to skip certificate verification.
    pub fn skip_cert_verification(&self) -> bool {
        self.skip_cert_verification
    }

    /// Get the client bind address for the connection.
    pub fn client_bind_addr(&self) -> &IpAddr {
        &self.client_bind_addr
    }

    /// Get whether to consider prior knowdlege of protocol support.
    pub fn prior_knowledge(&self) -> bool {
        self.prior_knowledge
    }
}

/// Trait that represents an HTTP connection.
pub trait HttpConnection {
    /// The sender to use.
    type Sender;

    /// Get the sender.
    fn sender(&mut self) -> &mut Self::Sender;
}

/// Trait that represents the HTTP connection pool.
pub trait HttpConnectionPool {
    /// The identity type.
    type Identity: crate::cert::Identity;
    /// The certificate type.
    type Certificate: crate::cert::Certificate;
    /// The connection dispatcher type.
    type ConnectionDispather: HttpConnectionDispatcher;
    /// The connection cache type.
    type ConnectionCache;

    /// Allow get connections.
    ///
    /// # Returns
    ///
    /// * `&Self::ConnectionCache` - The connections.
    ///
    fn connections(&self) -> &Self::ConnectionCache;

    /// Returns the number of connections.
    ///
    /// # Returns
    ///
    /// * `u32` - The number of connections.
    ///
    fn connection_count(&self) -> u32;

    /// Allow create a new connection.
    ///
    /// # Arguments
    ///
    /// * `config` - The connection configuration.
    /// * `dns_resolver` - The DNS resolver to use.
    ///
    /// # Returns
    ///
    /// * `Result<&mut Self::ConnectionDispather>` - The connection or error.
    ///
    fn create_connection<D>(
        &mut self,
        config: &ConnectionConfig<Self::Identity, Self::Certificate>,
        dns_resolver: &D,
    ) -> impl Future<Output = Result<&mut Self::ConnectionDispather>>
    where
        D: DnsResolver;
}

/// Trait that represents the HTTP connection dispatcher.
pub trait HttpConnectionDispatcher {
    /// Send a request through the connection.
    ///
    /// # Arguments
    ///
    /// * `request` - The request to send.
    ///
    /// # Returns
    ///
    /// * `Result<DeboaResponse>` - The response from the server.
    fn send_request(
        &mut self,
        request: Request<HttpBody>,
        timeout: Duration,
    ) -> impl Future<Output = Result<DeboaResponse>>;
}

/// Trait that represents the HTTP connection.
///
/// # Type Parameters
///
/// * `Connection` - The connection type.
/// * `RuntimeStream` - The runtime stream type.
///
pub trait ProtoConnection {
    /// The connection type.
    type Connection: HttpConnection;
    /// The runtime stream type.
    type RuntimeStream;

    /// Create a new connection.
    ///
    /// # Arguments
    ///
    /// * `stream` - The runtime stream to use.
    ///
    /// # Errors
    ///
    /// * `DeboaError` - If the connection fails.
    ///
    /// # Returns
    ///
    /// * `Result<Self::Connection>` - The connection or error.
    ///
    fn connect(stream: Self::RuntimeStream) -> impl Future<Output = Result<Self::Connection>>;

    /// Get connection protocol.
    ///
    /// # Returns
    ///
    /// * `Version` - The connection protocol.
    ///
    fn protocol_version(&self) -> Version;
}
