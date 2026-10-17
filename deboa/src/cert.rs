//! Certificate module
//!
//! This module provides functionality for handling client certificates and keys.
use std::{fmt::Debug, future::Future};

#[derive(Debug, Clone)]
/// Supported encodings for client certificates.
pub enum ContentEncoding {
    /// PEM encoding.
    PEM,
    /// DER encoding.
    DER,
}

/// Extension trait for Identity to provide native certificate loading methods.
pub trait IdentityNativeExt {
    /// Load a DER encoded PKCS#12 archive from a slice of bytes
    ///
    /// # Arguments
    ///
    /// * `bundle` - The DER encoded PKCS#12 archive.
    /// * `password` - The password for the PKCS#12 archive.
    ///
    /// # Returns
    ///
    /// * `Identity` - The new Identity instance.
    ///
    fn from_pkcs12(bundle: &[u8], password: Option<String>) -> Self;

    /// Load a DER encoded PKCS#12 archive from a file
    ///
    /// # Arguments
    ///
    /// * `file` - The path to the DER encoded PKCS#12 archive.
    /// * `password` - The password for the PKCS#12 archive.
    ///
    /// # Returns
    ///
    /// * `Identity` - The new Identity instance.
    ///
    fn from_pkcs12_file(
        file: &str,
        password: Option<String>,
    ) -> impl Future<Output = std::io::Result<Self>>
    where
        Self: Sized;
}

/// Extension trait for Identity to provide certificate loading methods.
pub trait IdentityExt {
    /// Load DER encoded certificate and key from a slice of bytes
    ///
    /// # Arguments
    ///
    /// * `cert` - The DER encoded certificate.
    /// * `key` - The DER encoded PKCS8 private key.
    /// * `encoding` - The encoding of the certificate and key.
    ///
    /// # Returns
    ///
    /// * `Identity` - The new Identity instance.
    fn from_pkcs8(cert: &[u8], key: &[u8], encoding: ContentEncoding) -> Self;

    /// Load DER encoded certificate and key from files
    ///
    /// # Arguments
    ///
    /// * `cert` - The path to the DER encoded certificate.
    /// * `key` - The path to the DER encoded PKCS8 private key.
    /// * `encoding` - The encoding of the certificate and key.
    ///
    /// # Returns
    ///
    /// * `Identity` - The new Identity instance.
    ///
    fn from_pkcs8_file(
        cert: &str,
        key: &str,
        encoding: ContentEncoding,
    ) -> impl Future<Output = std::io::Result<Self>>
    where
        Self: Sized;
}

/// Identity
pub trait Identity {
    /// Get the certificate
    ///
    /// # Returns
    ///
    /// * `&Vec<u8>` - The certificate
    fn cert(&self) -> &Vec<u8>;
    /// Get the private key
    ///
    /// # Returns
    ///
    /// * `&Option<Vec<u8>>` - The private key
    fn key(&self) -> Option<&Vec<u8>>;
    /// Get the encoding
    ///
    /// # Returns
    ///
    /// * `&Option<ContentEncoding>` - The encoding
    fn encoding(&self) -> Option<&ContentEncoding>;
}

/// Certificate
pub trait CertificateExt {
    /// Create certificate from slice of DER encoded bytes.
    ///
    /// # Arguments
    ///
    /// * `data` - The client certificate data.
    ///
    /// # Returns
    ///
    /// * `Certificate` - The new Certificate instance.
    ///
    fn from_slice(data: &[u8], encoding: ContentEncoding) -> Self;

    /// Create certificate from file of DER encoded file.
    ///
    /// # Arguments
    ///
    /// * `file` - The client certificate file path.
    ///
    /// # Returns
    ///
    /// * `Result<Certificate, std::io::Error>` - The new Certificate instance.
    ///
    fn from_file(
        file: &str,
        encoding: ContentEncoding,
    ) -> impl Future<Output = std::io::Result<Self>>
    where
        Self: Sized;
}

/// Certificate
pub trait Certificate {
    /// Allow get the client certificate path.
    ///
    /// # Returns
    ///
    /// * `&str` - The client certificate path.
    ///
    fn as_bytes(&self) -> &Vec<u8>;
}
