//! # HTTP Response Module
//!
//! This module provides comprehensive HTTP response handling capabilities for the Deboa HTTP client.
//! It includes response parsing, body processing, cookie management, and various utilities for
//! working with HTTP responses.
//!
//! ## Key Components
//!
//! - [`DeboaResponse`]: Main response structure with full HTTP functionality
//! - [`IntoBody`]: Trait for converting types into response bodies
//! - [`DeboaBody`]: Type alias for response body types
//! - Response body deserialization and streaming
//! - Cookie extraction and management
//! - Response status and header handling
//!
//! ## Features
//!
//! - Async response body streaming
//! - Automatic JSON deserialization
//! - Cookie jar integration
//! - Response body buffering and streaming
//! - Status code handling
//! - Header access and manipulation
//! - Response upgrade support (WebSocket, etc.)
//! - Runtime-agnostic body handling (Tokio/Smol)
//!
//! ## Examples
//!
//! ### Basic Response Handling
//!
//! ```rust, ignore
//! use deboa::{Client, request::IntoRequest};
//!
//! let mut client = Client::new();
//! let response = "https://api.example.com/data"
//!     .into_request()
//!     .execute(&mut client)
//!     .await?;
//!
//! println!("Status: {}", response.status());
//! println!("Body: {}", response.text().await?);
//! ```
//!
//! ### JSON Response Parsing
//!
//! ```rust, ignore
//! use deboa::{Client, request::get};
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct User {
//!     id: u32,
//!     name: String,
//! }
//!
//! let mut client = Client::new();
//! let response = get("https://api.example.com/user/1")
//!     .execute(&mut client)
//!     .await?;
//!
//! let user: User = response.json().await?;
//! println!("User: {}", user.name);
//! ```
//!
//! ### Streaming Response Body
//!
//! ```rust, ignore
//! use deboa::{Client, request::get};
//! use futures::StreamExt;
//!
//! let mut client = Client::new();
//! let response = get("https://api.example.com/large-data")
//!     .execute(&mut client)
//!     .await?;
//!
//! let mut stream = response.bytes_stream();
//! while let Some(chunk) = stream.next().await {
//!     // Process chunk
//! }
//! ```
use crate::{
    cookie::DeboaCookie,
    errors::{DeboaError, IoError},
    serde::ResponseBody,
    Result,
};
use http::{header, HeaderName, HeaderValue, Response};
use http_body_util::BodyExt;
use hyper_body_utils::HttpBody;
use log::error;
use serde::Deserialize;
use std::{fmt::Debug, fs::write};

/// Trait to allow converting a type into a DeboaBody.
///
/// This trait provides a flexible way to convert various input types into
/// HTTP response bodies. It enables convenient body creation from bytes,
/// strings, and other body-like objects.
///
/// # Examples
///
/// ``` compile_fail
/// use deboa::{Client, response::IntoBody};
///
/// let mut client = Client::new();
///
/// let response = b"Some bytes"
///   .into_body()
///   .unwrap();
/// assert_eq!(response, DeboaBody::Right(Full::<Bytes>::from(b"Some bytes")));
/// ```
pub trait IntoBody {
    /// Convert self to a HttpBody
    fn into_body(self) -> HttpBody;
}

impl IntoBody for &[u8] {
    #[inline]
    fn into_body(self) -> HttpBody {
        HttpBody::from_bytes(self)
    }
}

impl IntoBody for Vec<u8> {
    #[inline]
    fn into_body(self) -> HttpBody {
        HttpBody::from_bytes(&self)
    }
}

/// Deboa response builder
pub struct DeboaResponseBuilder {
    inner: Response<HttpBody>,
}

impl DeboaResponseBuilder {
    /// Allow set response version at any time.
    ///
    /// # Arguments
    ///
    /// * `version` - The new version.
    ///
    /// # Returns
    ///
    /// * `Self` - The response builder.
    ///
    pub fn version(mut self, version: http::Version) -> Self {
        *self
            .inner
            .version_mut() = version;
        self
    }

    /// Allow set response status at any time.
    ///
    /// # Arguments
    ///
    /// * `status` - The new status.
    ///
    /// # Returns
    ///
    /// * `Self` - The response builder.
    ///
    #[inline]
    pub fn status(mut self, status: http::StatusCode) -> Self {
        *self
            .inner
            .status_mut() = status;
        self
    }

    /// Allow set response headers at any time.
    ///
    /// # Arguments
    ///
    /// * `headers` - The new headers.
    ///
    /// # Returns
    ///
    /// * `Self` - The response builder.
    ///
    #[inline]
    pub fn headers(mut self, headers: http::HeaderMap) -> Self {
        *self
            .inner
            .headers_mut() = headers;
        self
    }

    /// Allow set response header at any time.
    ///
    /// # Arguments
    ///
    /// * `name` - The header name.
    /// * `value` - The header value.
    ///
    /// # Returns
    ///
    /// * `Self` - The response builder.
    ///
    #[inline]
    pub fn header(mut self, name: HeaderName, value: &str) -> Self {
        let header_value = HeaderValue::from_str(value);
        if let Ok(header_value) = header_value {
            self.inner
                .headers_mut()
                .insert(name, header_value);
        }
        self
    }

    /// Allow set response body at any time.
    ///
    /// # Arguments
    ///
    /// * `body` - The new body.
    ///
    /// # Returns
    ///
    /// * `Self` - The response builder.
    ///
    #[inline]
    pub fn body<B: IntoBody>(mut self, body: B) -> Self {
        let parts = self
            .inner
            .into_parts();
        self.inner = Response::from_parts(parts.0, body.into_body());
        self
    }

    /// Create an empty response.
    ///
    /// # Returns
    ///
    /// * `DeboaResponse` - The empty response.
    ///
    #[inline]
    pub fn empty(self) -> DeboaResponse {
        DeboaResponse { inner: self.inner }
    }

    /// Build the response. Consuming the builder.
    ///
    /// # Returns
    ///
    /// * `DeboaResponse` - The response.
    ///
    #[inline]
    pub fn build(self) -> DeboaResponse {
        DeboaResponse { inner: self.inner }
    }
}

/// Represents an HTTP response received from a server.
///
/// `DeboaResponse` provides methods to access and manipulate the response status,
/// headers, and body. It supports various ways to consume the response body,
/// including streaming and buffered access.
///
/// # Examples
///
/// ## Basic Usage
///
/// ```ignore
/// use deboa::{Client, request::get, Result};
///
/// # #[tokio::main]
/// async fn main() -> Result<()> {
///     let mut client = Client::new();
///     let response = get("https://httpbin.org/get")?
///       .send_with(&mut client)
///       .await?;
///
///     println!("Status: {}", response.status());
///     println!("Headers: {:?}", response.headers());
///     println!("Body: {}", response.text().await?);
///     Ok(())
/// }
/// ```
///
/// ## JSON Deserialization
///
/// ```compile_fail
/// use deboa::{Client, request::get, Result};
/// use deboa_extras::http::serde::json::JsonBody;
/// use serde::Deserialize;
///
/// #[derive(Debug, Deserialize)]
/// struct Data {
///     origin: String,
///     url: String,
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let mut client = Client::new();
///     let response = get("https://httpbin.org/get")?
///       .send_with(&mut client)
///       .await?;
///     let data: Data = response
///       .body_as(JsonBody)
///       .await?;
///     println!("Origin: {}", data.origin);
///     Ok(())
/// }
/// ```
///
/// # Fields
///
/// * `url` - The URL that the response came from
/// * `inner` - The underlying HTTP response
/// * `status` - The HTTP status code
/// * `headers` - The response headers
/// * `body` - The response body (can be streamed or buffered)
pub struct DeboaResponse {
    inner: Response<HttpBody>,
}

impl Debug for DeboaResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeboaResponse")
            .field("status", &self.inner.status())
            .field("headers", &self.inner.headers())
            .field("version", &self.inner.version())
            .finish()
    }
}

impl AsRef<DeboaResponse> for DeboaResponse {
    fn as_ref(&self) -> &DeboaResponse {
        self
    }
}

impl AsMut<DeboaResponse> for DeboaResponse {
    fn as_mut(&mut self) -> &mut DeboaResponse {
        self
    }
}

impl DeboaResponse {
    /// Allow create a new DeboaResponse instance.
    ///
    /// # Arguments
    ///
    /// * `url` - The url of the response.
    /// * `inner` - The inner response.
    ///
    pub fn new(inner: Response<HttpBody>) -> Self {
        Self { inner }
    }

    /// Create a new DeboaResponseBuilder
    #[inline]
    pub fn builder() -> DeboaResponseBuilder {
        DeboaResponseBuilder { inner: Response::new(HttpBody::from_bytes(&[])) }
    }

    /// Allow get version at any time.
    ///
    /// # Returns
    ///
    /// * `http::Version` - The version of the response.
    ///
    #[inline]
    pub fn version(&self) -> http::Version {
        self.inner.version()
    }

    /// Allow get mutable version at any time.
    ///
    /// # Returns
    ///
    /// * `&mut http::Version` - The version of the response.
    ///
    pub fn version_mut(&mut self) -> &mut http::Version {
        self.inner
            .version_mut()
    }

    /// Allow get status code at any time.
    ///
    /// # Returns
    ///
    /// * `http::StatusCode` - The status code of the response.
    ///
    #[inline]
    pub fn status(&self) -> http::StatusCode {
        self.inner.status()
    }

    /// Allow get mutable status code at any time.
    ///
    /// # Returns
    ///
    /// * `&mut http::StatusCode` - The status code of the response.
    ///
    #[inline]
    pub fn status_mut(&mut self) -> &mut http::StatusCode {
        self.inner
            .status_mut()
    }

    /// Allow get headers at any time.
    ///
    /// # Returns
    ///
    /// * `&http::HeaderMap` - The headers of the response.
    ///
    #[inline]
    pub fn headers(&self) -> &http::HeaderMap {
        self.inner.headers()
    }

    /// Allow get mutable headers at any time.
    ///
    /// # Returns
    ///
    /// * `&mut http::HeaderMap` - The headers of the response.
    ///
    #[inline]
    pub fn headers_mut(&mut self) -> &mut http::HeaderMap {
        self.inner
            .headers_mut()
    }

    /// Allow get header value at any time.
    /// It will return an error if the Content-Type header is missing or
    /// has an invalid value.
    ///
    /// # Arguments
    ///
    /// * `header` - The header name.
    ///
    /// # Returns
    ///
    /// * `Result<String>` - The header value.
    ///
    /// # Panics
    /// - If the header is missing
    /// - If the header value is invalid
    ///
    #[inline]
    fn header_value(&self, header: HeaderName) -> Result<String> {
        let header_name = header.as_str();
        let header_value = self
            .headers()
            .get(header_name);
        if header_value.is_none() {
            error!("Header {} is missing", header_name);
            return Err(DeboaError::Header { message: "Header is missing".to_string() });
        }
        let header_value = header_value.unwrap();
        let header_value = header_value.to_str();
        if let Err(e) = header_value {
            error!("Failed to read {}:: {}", header_name, e);
            return Err(DeboaError::Header {
                message: format!("Failed to read {}:: {}", header_name, e),
            });
        }
        Ok(header_value
            .unwrap()
            .to_string())
    }

    /// Allow get the length of the response body.
    /// It will return an error if the Content-Length header is missing or
    /// has an invalid value or if it fails to parse the value.
    ///
    /// # Returns
    ///
    /// * `Result<u64>` - The length of the response body.
    ///
    /// # Panics
    /// - If the Content-Length header is missing
    /// - If the Content-Length header value is invalid
    ///
    #[inline]
    pub fn content_length(&self) -> Result<u64> {
        let header = self.header_value(header::CONTENT_LENGTH)?;
        let header = header.parse::<u64>();
        if let Err(e) = header {
            error!("Failed to parse content-length: {}", e);
            return Err(DeboaError::Header {
                message: format!("Failed to parse content-length: {}", e),
            });
        }

        Ok(header.unwrap())
    }

    /// Allow get the content type of the response body.
    /// It will return an error if the Content-Type header is missing or
    /// has an invalid value.
    ///
    /// # Returns
    ///
    /// * `Result<String>` - The content type of the response body.
    ///
    /// # Panics
    /// - If the Content-Type header is missing
    /// - If the Content-Type header value is invalid
    ///
    #[inline]
    pub fn content_type(&self) -> Result<String> {
        let header = self.header_value(header::CONTENT_TYPE)?;
        Ok(header)
    }

    /// Retrieves cookies from response headers. If cookies are not found, returns None.
    /// Please note that this method will parse the cookies from the response headers.
    ///
    /// # Returns
    ///
    /// * `Option<Vec<DeboaCookie>>` - The cookies of the response.
    ///
    /// # Panics
    /// - If the Set-Cookie header is missing
    /// - If the Set-Cookie header value is invalid
    ///
    #[inline]
    pub fn cookies(&self) -> Result<Option<Vec<DeboaCookie>>> {
        let view = self
            .headers()
            .get_all(header::SET_COOKIE);
        let cookies = view
            .into_iter()
            .map(|cookie| {
                let cookie = cookie.to_str();
                if let Ok(cookie) = cookie {
                    DeboaCookie::parse_from_header(cookie)
                } else {
                    error!("Invalid cookie header");
                    Err(DeboaError::Cookie { message: "Invalid cookie header".to_string() })
                }
            })
            .collect::<Result<Vec<DeboaCookie>>>()
            .unwrap();

        if cookies.is_empty() {
            Ok(None)
        } else {
            Ok(Some(cookies))
        }
    }

    /// Allow get inner response at any time.
    ///
    /// # Returns
    ///
    /// * `DeboaBody` - The inner response.
    ///
    #[inline]
    pub fn into_inner(self) -> Response<HttpBody> {
        self.inner
    }

    /// Allow get inner response body at any time.
    ///
    /// # Returns
    ///
    /// * `DeboaBody` - The inner response body.
    ///
    #[inline]
    pub fn inner_body(self) -> HttpBody {
        self.inner
            .into_body()
    }

    /// Allow get stream body at any time.
    ///
    /// # Returns
    ///
    /// * `Either<Incoming, Full<Bytes>>` - The stream body of the response.
    ///
    #[inline]
    pub fn stream(self) -> HttpBody {
        self.inner
            .into_body()
    }

    /// Allow get inner response parts at any time.
    ///
    /// # Returns
    ///
    /// * `http::response::Parts` - The parts of the response.
    /// * `DeboaBody` - The body of the response.
    ///
    /// # Example
    ///
    /// ```compile_fail
    /// let (parts, body) = response.into_parts();
    /// ```
    #[inline]
    pub fn into_parts(self) -> (http::response::Parts, HttpBody) {
        let (parts, body) = self
            .inner
            .into_parts();
        (parts, body)
    }

    /// Returns the response body as a deserialized type, consuming body.
    /// Useful for small responses. For larger responses, consider using `stream`.
    ///
    /// # Arguments
    ///
    /// * `body_type` - The body type to be deserialized.
    ///
    /// # Returns
    ///
    /// * `Result<B>` - The body or error.
    ///
    /// # Examples
    ///
    /// ```compile_fail
    /// use deboa::request::get;
    /// use deboa_extras::http::serde::json::JsonBody;
    ///
    /// let response = get("https://jsonplaceholder.typicode.com/posts")?
    ///     .send_with(client)
    ///     .await?;
    /// let posts: Vec<Post> = response
    ///     .body_as(JsonBody)
    ///     .await?;
    /// ```
    ///
    #[inline]
    pub async fn body_as<R: ResponseBody, B: for<'a> Deserialize<'a>>(
        self,
        body_type: R,
    ) -> Result<B> {
        let bytes = self.bytes().await?;
        let result = body_type.deserialize::<B>(bytes)?;
        Ok(result)
    }

    /// Returns the response body as a string, consuming body.
    /// Useful for small responses. For larger responses, consider using `stream`.
    ///
    /// # Returns
    ///
    /// * `Result<String>` - The text body or error.
    ///
    /// # Examples
    ///
    /// ```compile_fail
    /// use deboa::request::get;
    ///
    /// let response = get("https://jsonplaceholder.typicode.com/posts")?
    ///     .send_with(client)
    ///     .await?;
    /// let text = response
    ///     .text()
    ///     .await?;
    /// ```
    ///
    #[inline]
    pub async fn text(self) -> Result<String> {
        let body = self.bytes().await?;
        Ok(String::from_utf8_lossy(&body).to_string())
    }

    /// Save response body to file, consuming body.
    /// Useful for small responses. For larger responses, consider using
    /// ToFile trait available on utils feature of deboa-extras crate.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to save the file.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - The result or error.
    ///
    /// # Examples
    ///
    /// ```compile_fail
    /// use deboa::request::get;
    ///
    /// let response = get("https://jsonplaceholder.typicode.com/posts")?
    ///     .send_with(client)
    ///     .await?;
    /// response
    ///     .to_file("posts.json")
    ///     .await?;
    /// ```
    ///
    #[inline]
    pub async fn to_file(self, path: &str) -> Result<()> {
        let body = self.bytes().await?;
        let result = write(path, body);
        if let Err(e) = result {
            error!("Failed to write file: {}", e);
            return Err(DeboaError::Io(IoError::File { message: e.to_string() }));
        }
        Ok(())
    }

    /// Convenient alias to raw_body.
    ///
    /// # Returns
    ///
    /// * `Vec<u8>` - The raw body of the response.
    ///
    #[inline]
    pub async fn bytes(self) -> Result<Vec<u8>> {
        let mut data = Vec::<u8>::new();
        let bytes = self
            .inner_body()
            .collect()
            .await;
        match bytes {
            Ok(bytes) => data.extend_from_slice(&bytes.to_bytes()),
            Err(e) => {
                error!("Failed to collect response body: {}", e);
                return Err(DeboaError::Io(IoError::Content { message: e.to_string() }));
            }
        }
        Ok(data)
    }
}
