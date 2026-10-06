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

use async_fn_stream::StreamEmitter;
use bytes::{Bytes, BytesMut};
use futures::StreamExt;
use http_body::Frame;
use http_body_util::Full;
use hyper_body_utils::HttpBody;
use rand::distr::{Alphanumeric, SampleString};
use std::{
    fmt::Write,
    io::{Error, ErrorKind::InvalidData},
    pin::Pin,
};
use urlencoding::encode;

pub(crate) const ENCODED_FORM_TYPE: &str = "application/x-www-form-urlencoded";
pub(crate) const MULTIPART_FORM_TYPE: &str = "multipart/form-data";
pub(crate) const CRLF: &str = "\r\n";

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

/// Type to hold a stream of binary content
pub struct MimeStream {
    name: String,
    file_name: String,
    mime_type: String,
    inner: Pin<Box<dyn futures::Stream<Item = Result<Bytes, std::io::Error>>>>,
}

/// Form parts
pub enum Part {
    /// String part
    String((String, String)),
    /// Streeam part (files)
    Stream(MimeStream),
}

async fn render_string(
    emitter: &StreamEmitter<Result<Frame<Bytes>, std::io::Error>>,
    key: &str,
    value: &str,
) {
    let mut bytes = BytesMut::new();
    match write!(
        &mut bytes,
        "Content-Disposition: form-data; name=\"{key}\"{CRLF}{CRLF}{}{CRLF}",
        &value
    ) {
        Ok(()) => {
            emitter
                .emit(Ok(Frame::data(bytes.into())))
                .await
        }
        Err(e) => {
            emitter
                .emit(Err(std::io::Error::new(InvalidData, e)))
                .await
        }
    };
}

async fn render_stream(
    emitter: &StreamEmitter<Result<Frame<Bytes>, std::io::Error>>,
    stream: &mut MimeStream,
) {
    let mut bytes = BytesMut::new();
    match write!(&mut bytes,
            "Content-Disposition: form-data; name=\"{}\"; filename=\"{}\"{CRLF}Content-Type: {}{CRLF}{CRLF}",
            stream.name, stream.file_name, stream.mime_type
    ) {
        Ok(()) => {
            emitter
                .emit(Ok(Frame::data(bytes.into())))
                .await
        }
        Err(e) => {
            emitter
                .emit(Err(std::io::Error::new(InvalidData, e)))
                .await
        }
    }

    while let Some(chunk) = stream
        .inner
        .next()
        .await
    {
        emitter
            .emit(chunk.map(|val| Frame::data(val)))
            .await;
    }
    emitter
        .emit(Ok(Frame::data(CRLF.into())))
        .await;
}

impl Part {
    /// Create a string part
    pub fn string(key: &str, value: &str) -> Part {
        Part::String((key.into(), value.into()))
    }

    /// Create a stream part
    pub fn stream(stream: MimeStream) -> Part {
        Part::Stream(stream)
    }

    /// Render parts
    pub async fn render(&mut self, emitter: &StreamEmitter<Result<Frame<Bytes>, std::io::Error>>) {
        match self {
            Part::String((key, value)) => {
                render_string(emitter, key, value).await;
            }
            Part::Stream(stream) => {
                render_stream(emitter, stream).await;
            }
        }
    }
}

/// Multipart form
pub struct MultiPartForm {
    fields: Vec<Part>,
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
    #[inline]
    /// Create a new multi part form.
    ///
    /// # Returns
    ///
    /// * `Self` - The multi part form.
    pub fn builder() -> Self {
        let boundary = Alphanumeric.sample_string(&mut rand::rng(), 10);
        Self { fields: Vec::new(), boundary: format!("DeboaFormBdry{}", boundary) }
    }

    #[inline]
    /// Returns form content-type
    ///
    /// # Returns
    ///
    /// * `String` - Form content-type
    pub fn content_type(&self) -> String {
        format!("{}; boundary={}", MULTIPART_FORM_TYPE, self.boundary)
    }

    #[inline]
    /// Get the boundary of the form.
    ///
    /// # Returns
    ///
    /// * `String` - The boundary.
    pub fn boundary(&self) -> String {
        self.boundary
            .to_string()
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
    pub fn field(mut self, name: &str, value: &str) -> Self {
        self.fields
            .push(Part::String((name.into(), value.into())));
        self
    }

    #[inline]
    /// Add a stream to the form.
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
    pub fn stream<S>(mut self, name: &str, file_name: &str, mime_type: &str, stream: S) -> Self
    where
        S: futures::Stream<Item = Result<Bytes, std::io::Error>> + 'static,
    {
        self.fields
            .push(Part::Stream(MimeStream {
                name: name.into(),
                file_name: file_name.into(),
                mime_type: mime_type.into(),
                inner: Box::pin(stream),
            }));
        self
    }

    /// Build the form, returning it as a stream of Frame<Bytes>
    ///
    /// # Returns
    ///
    /// * `impl futures::Stream<Item = Result<Frame<Bytes>, Error>>` - Form content as a stream of Frame<Bytes>
    ///
    /// # Notes
    ///
    /// As opposite to EncodedForm, multipart doesn't have a quick way to convert to a HttṕBody
    /// you need call [http-body-utils::HttpBody::stream()]
    pub async fn build(mut self) -> impl futures::Stream<Item = Result<Frame<Bytes>, Error>> {
        let boundary = self.boundary;
        async_fn_stream::fn_stream(|emitter| async move {
            let mut bytes = BytesMut::new();
            match write!(&mut bytes, "--{}{CRLF}", &boundary) {
                Ok(()) => {
                    emitter
                        .emit(Ok(Frame::data(bytes.into())))
                        .await
                }
                Err(e) => {
                    emitter
                        .emit(Err(std::io::Error::new(InvalidData, e)))
                        .await
                }
            }

            let count = self.fields.len();
            for (index, part) in self
                .fields
                .iter_mut()
                .enumerate()
            {
                part.render(&emitter)
                    .await;

                let mut bytes = BytesMut::new();

                let ending = if index != count - 1 { CRLF } else { "--\r\n" };
                match write!(&mut bytes, "--{}{ending}", &boundary) {
                    Ok(()) => {
                        emitter
                            .emit(Ok(Frame::data(bytes.into())))
                            .await
                    }
                    Err(e) => {
                        emitter
                            .emit(Err(std::io::Error::new(InvalidData, e)))
                            .await
                    }
                }
            }
        })
    }
}
