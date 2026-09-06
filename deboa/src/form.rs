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
//! - [`Form`]: Common trait for building and encoding form data
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
//!     .file("avatar", "path/to/avatar.jpg")?;
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

use std::path::Path;

use bytes::{Bytes, BytesMut};
use indexmap::IndexMap;
use rand::distr::{Alphanumeric, SampleString};
use urlencoding::encode;

pub(crate) const CRLF: &[u8] = b"\r\n";

/// A trait for building and encoding form data for HTTP requests.
///
/// This trait provides a common interface for different types of form data,
/// including URL-encoded forms and multipart forms. It allows adding form fields
/// and building the final encoded representation.
///
/// # Implementations
///
/// - `EncodedForm`: For `application/x-www-form-urlencoded` form data
/// - `MultiPartForm`: For `multipart/form-data` form data, including file uploads
///
/// # Examples
///
/// ## URL-encoded Form
///
/// ```compile_fail
/// use deboa::form::EncodedForm;
///
/// let mut form = EncodedForm::builder()
///     .field("username", "user123")
///     .field("password", "s3cr3t");
///
/// let encoded = form.build();
/// ```
///
/// ## Multipart Form with File
///
/// ```compile_fail
/// use deboa::form::MultiPartForm;
///
/// let form = MultiPartForm::builder()
///     .field("name", "deboa")
///     .field("version", "1.0.0")
///     .file("avatar", "/path/to/avatar.jpg");
/// ```
pub trait DeboaForm {
    /// Get the content type of the form.
    ///
    /// # Returns
    ///
    /// * `String` - The content type.
    ///
    fn content_type(&self) -> String;
    /// Add a field to the form.
    ///
    /// # Arguments
    ///
    /// * `key` - The key.
    /// * `value` - The value.
    ///
    /// # Returns
    ///
    /// * `&mut Self` - The form.
    ///
    fn field(self, key: &str, value: &str) -> Self;
    /// Build the form.
    ///
    /// # Returns
    ///
    /// * `Bytes` - The encoded form.
    ///
    fn build(self) -> Bytes;
}

/// Enum that represents the form.
///
/// # Variants
///
/// * `EncodedForm` - The encoded form.
/// * `MultiPartForm` - The multi part form.
#[derive(Debug)]
pub enum Form {
    /// Encoded form
    EncodedForm(EncodedForm),
    /// Multi part form
    MultiPartForm(MultiPartForm),
}

impl From<EncodedForm> for Form {
    #[inline]
    fn from(val: EncodedForm) -> Self {
        Form::EncodedForm(val)
    }
}

impl From<MultiPartForm> for Form {
    #[inline]
    fn from(val: MultiPartForm) -> Self {
        Form::MultiPartForm(val)
    }
}

/// Encoded form
#[derive(Debug, Clone)]
pub struct EncodedForm {
    fields: IndexMap<String, String>,
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
/// use deboa::form::MultiPartForm;
///
/// let mut client = Deboa::default();
/// let mut form = MultiPartForm::builder();
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
    /// Create a new encoded form.
    ///
    /// # Returns
    ///
    /// * `Self` - The encoded form.
    ///
    pub fn builder() -> Self {
        Self { fields: IndexMap::new() }
    }
}

impl DeboaForm for EncodedForm {
    #[inline]
    fn content_type(&self) -> String {
        "application/x-www-form-urlencoded".to_string()
    }

    #[inline]
    fn field(mut self, key: &str, value: &str) -> Self {
        self.fields
            .insert(key.to_string(), value.to_string());
        self
    }

    #[inline]
    fn build(self) -> Bytes {
        self.fields
            .iter()
            .map(|(key, value)| format!("{}={}", key, encode(value)))
            .collect::<Vec<String>>()
            .join("&")
            .into_bytes()
            .into()
    }
}

/// Multi part form
#[derive(Debug, Clone)]
pub struct MultiPartForm {
    fields: IndexMap<String, String>,
    boundary: String,
}

/// Implement the builder pattern for MultiPartForm.
///
/// # Returns
///
/// * `Self` - The multi part form.
///
/// # Examples
///
/// ```compile_fail
/// use deboa::form::MultiPartForm;
///
/// let mut client = Deboa::default();
/// let mut form = MultiPartForm::builder();
/// form.field("name", "deboa");
/// form.field("version", "0.0.1");
///
/// let request = DeboaRequest::post("https://example.com/register")?
///     .form(form.into())
///     .build()?;
///
/// let mut response = client.execute(request).await?;
/// ```
impl MultiPartForm {
    /// Create a new multi part form.
    ///
    /// # Returns
    ///
    /// * `Self` - The multi part form.
    ///
    #[inline]
    pub fn builder() -> Self {
        let boundary = Alphanumeric.sample_string(&mut rand::rng(), 10);
        Self { fields: IndexMap::new(), boundary: format!("DeboaFormBdry{}", boundary) }
    }

    /// Add a file to the form.
    ///
    /// # Arguments
    ///
    /// * `key` - The key.
    /// * `value` - The value.
    ///
    /// # Returns
    ///
    /// * `&mut Self` - The form.
    ///
    #[inline]
    pub fn file<F>(mut self, key: &str, value: F) -> Self
    where
        F: AsRef<std::path::Path>,
    {
        self.fields.insert(
            key.to_string(),
            value
                .as_ref()
                .to_str()
                .unwrap()
                .to_string(),
        );
        self
    }

    /// Get the boundary of the form.
    ///
    /// # Returns
    ///
    /// * `String` - The boundary.
    ///
    #[inline]
    pub fn boundary(&self) -> String {
        self.boundary
            .to_string()
    }
}

impl DeboaForm for MultiPartForm {
    #[inline]
    fn content_type(&self) -> String {
        format!("multipart/form-data; boundary={}", self.boundary)
    }

    #[inline]
    fn field(mut self, key: &str, value: &str) -> Self {
        self.fields
            .insert(key.to_string(), value.to_string());
        self
    }

    fn build(self) -> Bytes {
        let mut form = BytesMut::new();
        let boundary = &self.boundary;
        form.extend_from_slice(b"--");
        form.extend_from_slice(boundary.as_bytes());
        form.extend_from_slice(CRLF);
        for (key, value) in &self.fields {
            if Path::is_file(value.as_ref()) {
                let kind = minimime::lookup_by_filename(value);
                let path = Path::new(value);
                if let Some(kind) = kind {
                    let file_name = path.file_name();
                    let file_content = std::fs::read(value).unwrap();
                    if let Some(file_name) = file_name {
                        form.extend_from_slice(
                            &format!(
                                "Content-Disposition: form-data; name=\"{}\"; filename=\"{}\"",
                                key,
                                file_name
                                    .to_str()
                                    .unwrap()
                            )
                            .into_bytes(),
                        );
                        form.extend_from_slice(CRLF);
                        form.extend_from_slice(
                            &format!("Content-Type: {}\r\n", kind.content_type).into_bytes(),
                        );
                        form.extend_from_slice(CRLF);
                        form.extend_from_slice(&file_content);
                        form.extend_from_slice(CRLF);
                    }
                }
            } else {
                form.extend_from_slice(
                    &format!("Content-Disposition: form-data; name=\"{}\"", key).into_bytes(),
                );
                form.extend_from_slice(CRLF);
                form.extend_from_slice(CRLF);
                form.extend_from_slice(value.as_bytes());
                form.extend_from_slice(CRLF);
            }

            form.extend_from_slice(b"--");
            form.extend_from_slice(boundary.as_bytes());

            if key
                != self
                    .fields
                    .last()
                    .unwrap()
                    .0
            {
                form.extend_from_slice(CRLF);
            } else {
                form.extend_from_slice(b"--\r\n");
            }
        }

        form.into()
    }
}
