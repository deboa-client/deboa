//! # HTTP Form Data Module
//!
//! This module provides comprehensive form data handling capabilities for HTTP requests.
//! It supports both URL-encoded forms and multipart forms, enabling easy submission
//! of form data including file uploads.
//!
//! ## Form Types
//!
//! - **URL-encoded Forms** (`application/x-www-form-urlencoded`): Standard form encoding
//!   for simple key-value pairs
//! - **Multipart Forms** (`multipart/form-data`): Supports file uploads and complex data
//!
//! ## Key Components
//!
//! - [`EncodedForm`]: URL-encoded form implementation
//! - [`MultiPartForm`]: Multipart form implementation with file upload support
//! - Form builders for fluent API usage
//!
//! ## Features
//!
//! - Type-safe form field addition
//! - Automatic URL encoding for form fields
//! - File upload support in multipart forms
//! - Boundary generation for multipart data
//! - Memory-efficient encoding using `Bytes` and `BytesMut`
//!
//! ## Examples
//!
//! ### URL-encoded Form
//!
//! ```rust, ignore
//! use deboa::form::EncodedForm;
//!
//! let mut form = EncodedForm::builder()
//!     .field("username", "user123")
//!     .field("password", "s3cr3t");
//!
//! let encoded = form.build();
//! ```
//!
//! ### Multipart Form with File Upload
//!
//! ```rust, ignore
//! use deboa::form::MultiPartForm;
//!
//! let mut form = MultiPartForm::builder()
//!     .field("description", "User profile")
//!     .file("avatar", "path/to/avatar.jpg", "image/jpeg", stream)?;
//!
//! let encoded = form.build();
//! ```
//!
//! ## Usage in HTTP Requests
//!
//! ```rust, ignore
//! use deboa::{Deboa, request::post, form::EncodedForm};
//!
//! let mut client = Deboa::default();
//! let form = EncodedForm::builder()
//!     .field("name", "John")
//!     .field("email", "john@example.com")
//!     .build();
//!
//! let response = post("https://api.example.com/submit")
//!     .form(form)?
//!     .execute(&mut client)
//!     .await?;
//! ```

use crate::{errors::DeboaError, request::DeboaRequest};
use bytes::Bytes;
use http::Method;
use http_body_util::Full;
use hyper_body_utils::HttpBody;
use url::Url;
use urlencoding::encode;

pub(self) const ENCODED_FORM_TYPE: &str = "application/x-www-form-urlencoded";

/// Encoded form
#[derive(Debug, Clone)]
pub struct EncodedForm {
    fields: Vec<(String, String)>,
}

/// Implement the builder pattern for EncodedForm.
///
/// # Returns
///
/// * `Self` - The encoded form.
///
/// # Examples
///
/// ```compile_fail
/// use deboa::form::EncodedForm;
///
/// let mut client = Deboa::default();
/// let mut form = EncodedForm::builder();
/// form.field("name", "deboa");
/// form.field("version", "0.0.1");
///
/// let request = DeboaRequest::post("https://example.com/register")?
///     .form(form.into())
///     .build()?;
///
/// let mut response = client.execute(request).await?;
/// ```
impl EncodedForm {
    #[inline]
    /// Create a new encoded form.
    ///
    /// # Returns
    ///
    /// * `Self` - The encoded form.
    ///
    pub fn builder() -> Self {
        Self { fields: Vec::new() }
    }

    #[inline]
    /// Returns form content-type
    ///
    /// # Returns
    ///
    /// * `String` - Form content-type
    pub fn content_type(&self) -> &str {
        ENCODED_FORM_TYPE
    }

    #[inline]
    /// Allow add a new form field, a string one mainly
    ///
    /// # Arguments
    ///
    /// * `key` - Field name
    /// * `value` - Field content
    ///
    /// # Returns
    ///
    /// * `Self` - Form reference for chained calls
    pub fn field(mut self, key: &str, value: &str) -> Self {
        self.fields
            .push((key.into(), value.into()));
        self
    }

    #[inline]
    /// Build the form
    ///
    /// # Returns
    ///
    /// * `Bytes` - Form content as Bytes
    pub fn build(self) -> Bytes {
        self.fields
            .iter()
            .map(|(key, value)| format!("{}={}", key, encode(value)))
            .collect::<Vec<String>>()
            .join("&")
            .into_bytes()
            .into()
    }
}

impl From<EncodedForm> for HttpBody {
    #[inline]
    fn from(val: EncodedForm) -> Self {
        HttpBody::Standard(Full::new(val.build()))
    }
}

impl TryFrom<(Url, Method, EncodedForm)> for DeboaRequest {
    type Error = DeboaError;

    fn try_from(value: (Url, Method, EncodedForm)) -> Result<Self, Self::Error> {
        DeboaRequest::from(value.0)?
            .method(value.1)
            .bytes(
                &value
                    .2
                    .build()
                    .to_vec(),
            )
            .build()
    }
}
