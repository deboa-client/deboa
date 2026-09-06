//! # HTTP Request Module
//!
//! This module provides comprehensive HTTP request building and handling functionality
//! for the Deboa HTTP client. It includes traits and structs for creating, configuring,
//! and executing HTTP requests with various features like authentication, headers,
//! cookies, and body serialization.
//!
//! ## Key Components
//!
//! - [`IntoRequest`]: Trait for converting various types into HTTP requests
//! - [`IntoHeaders`]: Trait for converting various types into HTTP headers
//! - [`DeboaRequest`]: Main request structure with full HTTP functionality
//! - Request builders for different HTTP methods (GET, POST, PUT, DELETE, etc.)
//! - Authentication mechanisms (Basic, Bearer token, custom)
//! - Header and cookie management
//! - Form data and JSON serialization support
//!
//! ## Features
//!
//! - Type-safe request building
//! - Automatic content-type handling
//! - Authentication support (Basic, Bearer, custom)
//! - Cookie jar integration
//! - Form data and JSON serialization
//! - File upload support
//! - Request retry mechanisms
//! - Custom headers and query parameters
//!
//! ## Examples
//!
//! ### Basic GET Request
//!
//! ```rust, ignore
//! use deboa::{request::IntoRequest};
//! use deboa_tokio::Client;
//!
//! let mut client = Client::new();
//! let response = "https://api.example.com/data".into_request().execute(&mut client).await?;
//! ```
//!
//! ### POST Request with JSON
//!
//! ```rust, ignore
//! use deboa::{request::post};
//! use deboa_extras::http::serde::json::JsonBody;
//! use deboa_tokio::Client;
//!
//! let mut client = Client::new();
//! let response = post("https://api.example.com/users")
//!     .body_as(JsonBody, json!({"name": "John", "age": 30}))?
//!     .send_with(&mut client)
//!     .await?;
//! ```
//!
//! ### Authentication
//!
//! ```rust, ignore
//! use deboa::{request::get};
//! use deboa_tokio::Client;
//!
//! let mut client = Client::new();
//! let response = get("https://api.example.com/protected")
//!     .basic_auth("username", "password")
//!     .send_with(&mut client)
//!     .await?;
//! ```

use crate::{
    cert::{Certificate, Identity},
    conn::HttpConnectionPool,
    cookie::DeboaCookie,
    dns::DnsResolver,
    errors::{DeboaError, RequestError},
    form::{DeboaForm, Form},
    response::DeboaResponse,
    serde::RequestBody,
    url::IntoUrl,
    Client, HttpClient, InnerClient, Result,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use bytes::Bytes;
use hashbrown::HashMap;
use http::{
    header::{self},
    HeaderMap, HeaderName, HeaderValue, Method, Request, Uri, Version,
};
use http_body_util::combinators::BoxBody;
use hyper_body_utils::HttpBody;
use log::error;
use regex::Regex;
use serde::Serialize;
use std::{fmt::Debug, future::Future, str::FromStr};
use url::Url;

/// Bytes body type
pub type BytesBody = BoxBody<Bytes, std::io::Error>;
/// HTTP/1 request type
pub type Http1Request = hyper::client::conn::http1::SendRequest<HttpBody>;
/// HTTP/2 request type
pub type Http2Request = hyper::client::conn::http2::SendRequest<HttpBody>;

/// Trait to allow making a request from different types.
///
/// This trait provides a flexible way to convert various input types into
/// HTTP requests. It enables convenient request creation from strings, URLs,
/// and other request-like objects.
///
/// # Examples
///
/// ```rust,compile_fail
/// use deboa::{request::IntoRequest};
/// use deboa_tokio::Client;
///
/// let mut client = Client::new();
///
/// let response = "https://jsonplaceholder.typicode.com"
///   .into_request()
///   .await?;
/// assert_eq!(response.status(), 200);
/// ```
pub trait IntoRequest: private::IntoRequestSealed {
    /// Convert self to a DeboaRequest
    fn into_request(self) -> Result<DeboaRequest>;
}

impl IntoRequest for DeboaRequest {
    #[inline]
    fn into_request(self) -> Result<DeboaRequest> {
        Ok(self)
    }
}

impl IntoRequest for &str {
    #[inline]
    fn into_request(self) -> Result<DeboaRequest> {
        DeboaRequest::get(self)?.build()
    }
}

impl IntoRequest for String {
    #[inline]
    fn into_request(self) -> Result<DeboaRequest> {
        DeboaRequest::get(self)?.build()
    }
}

impl IntoRequest for Url {
    #[inline]
    fn into_request(self) -> Result<DeboaRequest> {
        DeboaRequest::get(self)?.build()
    }
}

/// Trait to allow adding headers to a request.
///
/// This trait provides a flexible way to convert various input types into
/// HTTP headers.
///
/// # Examples
///
/// ```rust,compile_fail
/// use deboa::request::{IntoHeaders, DeboaRequest, DeboaRequestBuilder};
///
/// let headers = vec![("User-Agent", "deboa/0.1")];
/// let request = DeboaRequest::get("https://example.com")?
///     .headers(headers)
///     .build()?;
/// ```
pub trait IntoHeaders: private::IntoHeadersSealed {
    /// Convert self to a HeaderMap
    fn into_headers(self) -> Result<HeaderMap>;
}

impl IntoHeaders for HeaderMap {
    #[inline]
    fn into_headers(self) -> Result<HeaderMap> {
        Ok(self)
    }
}

impl IntoHeaders for Vec<(HeaderName, String)> {
    #[inline]
    fn into_headers(self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        for (key, value) in self {
            headers.insert(&key, HeaderValue::from_str(&value).expect("Invalid header value"));
        }
        Ok(headers)
    }
}

impl IntoHeaders for Vec<(String, String)> {
    #[inline]
    fn into_headers(self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        for (key, value) in self {
            headers.insert(
                HeaderName::from_str(&key).expect("Invalid header name"),
                HeaderValue::from_str(&value).expect("Invalid header value"),
            );
        }
        Ok(headers)
    }
}

impl<'a> IntoHeaders for Vec<(&'a str, &'a str)> {
    #[inline]
    fn into_headers(self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        for (key, value) in self {
            headers.insert(
                HeaderName::from_str(key).expect("Invalid header name"),
                HeaderValue::from_str(value).expect("Invalid header value"),
            );
        }
        Ok(headers)
    }
}

/// Extension trait for HTTP methods to create requests.
/// Allows creating requests using method names as strings or Method enum values.
///
/// # Examples
/// ```rust,compile_fail
/// use http::Method;
/// use deboa::request::MethodExt;
///
/// // Using Method enum
/// let request = Method::GET.from_url("https://example.com")?;
///
/// // Using string
/// let request = "GET".from_url("https://example.com")?;
/// ```
pub trait MethodExt: private::MethodExtSealed {
    /// Create a request from a URL
    fn from_url(self, url: &str) -> Result<DeboaRequestBuilder>;
    /// Create a request to a URL
    fn to_url(self, url: &str) -> Result<DeboaRequestBuilder>;
}

impl MethodExt for Method {
    fn from_url(self, url: &str) -> Result<DeboaRequestBuilder> {
        match self {
            Method::GET => DeboaRequest::get(url),
            Method::POST => DeboaRequest::post(url),
            Method::QUERY => DeboaRequest::query(url),
            Method::PUT => DeboaRequest::put(url),
            Method::DELETE => DeboaRequest::delete(url),
            Method::PATCH => DeboaRequest::patch(url),
            _ => panic!("Method not supported"),
        }
    }

    fn to_url(self, url: &str) -> Result<DeboaRequestBuilder> {
        self.from_url(url)
    }
}

impl MethodExt for &str {
    #[inline]
    fn from_url(self, url: &str) -> Result<DeboaRequestBuilder> {
        match self {
            "GET" | "get" => DeboaRequest::get(url),
            "POST" | "post" => DeboaRequest::post(url),
            "QUERY" | "query" => DeboaRequest::query(url),
            "PUT" | "put" => DeboaRequest::put(url),
            "DELETE" | "delete" => DeboaRequest::delete(url),
            "PATCH" | "patch" => DeboaRequest::patch(url),
            _ => panic!("Method not supported"),
        }
    }

    #[inline]
    fn to_url(self, url: &str) -> Result<DeboaRequestBuilder> {
        self.from_url(url)
    }
}

/// Trait to allow make a get request from different types.
///
/// # Examples
///
/// ```rust,compile_fail
/// use deboa::{Deboa, request::FetchWith};
///
/// let client = Deboa::default();
///
/// let response = "https://jsonplaceholder.typicode.com"
///   .fetch_with(&client)
///   .await?;
/// assert_eq!(response.status(), 200);
/// ```
pub trait FetchWith {
    /// Fetch the request.
    ///
    /// # Returns
    ///
    /// * `Result<DeboaResponse>` - The response.
    ///
    /// # Examples
    ///
    /// ```rust,compile_fail
    /// use deboa::{request::FetchWith};
    /// use deboa_tokio::Client;
    ///
    /// let client = Client::new();
    ///
    /// let response = "https://jsonplaceholder.typicode.com"
    ///   .fetch_with(&client)
    ///   .await?;
    /// assert_eq!(response.status(), 200);
    /// ```
    ///
    fn fetch_with<I, C, P, R>(
        &self,
        client: &Client<InnerClient<I, C, P, R>>,
    ) -> impl Future<Output = Result<DeboaResponse>>
    where
        I: Identity + Send + Clone,
        C: Certificate + Send + Clone,
        P: HttpConnectionPool<Identity = I, Certificate = C> + Send,
        R: DnsResolver + Send;
}

impl FetchWith for &str {
    #[inline]
    async fn fetch_with<I, C, P, R>(
        &self,
        client: &Client<InnerClient<I, C, P, R>>,
    ) -> Result<DeboaResponse>
    where
        I: Identity + Send + Clone,
        C: Certificate + Send + Clone,
        P: HttpConnectionPool<Identity = I, Certificate = C> + Send,
        R: DnsResolver + Send,
    {
        DeboaRequest::get(*self)?
            .send_with(client)
            .await
    }
}

impl FetchWith for String {
    #[inline]
    async fn fetch_with<I, C, P, R>(
        &self,
        client: &Client<InnerClient<I, C, P, R>>,
    ) -> Result<DeboaResponse>
    where
        I: Identity + Send + Clone,
        C: Certificate + Send + Clone,
        P: HttpConnectionPool<Identity = I, Certificate = C> + Send,
        R: DnsResolver + Send,
    {
        DeboaRequest::get(self)?
            .send_with(client)
            .await
    }
}

/// A utility function to create a GET request within DeboaRequest.
///
/// # Arguments
///
/// * `url` - The url to connect.
///
/// # Returns
///
/// * `Result<DeboaRequestBuilder>` - The request builder.
///
/// # Examples
///
/// ```rust,compile_fail
/// use deboa::{request::get};
/// use deboa_tokio::Client;
///
/// let client = Client::new();
///
/// let request = get("https://jsonplaceholder.typicode.com").unwrap();
/// let response = request.send_with(&client).await?;
/// assert_eq!(response.status(), 200);
/// ```
///
#[inline]
pub fn get<T: IntoUrl>(url: T) -> Result<DeboaRequestBuilder> {
    DeboaRequest::get(url)
}

/// A utility function to create a POST request within DeboaRequest.
///
/// # Arguments
///
/// * `url` - The url to connect.
///
/// # Returns
///
/// * `Result<DeboaRequestBuilder>` - The request builder.
///
/// # Examples
///
/// ```rust,compile_fail
/// use deboa::{request::post};
/// use deboa_tokio::Client;
///
/// let client = Client::new();
///
/// let request = post("https://jsonplaceholder.typicode.com/posts")?
///   .raw_body(b"{\"title\": \"foo\", \"body\": \"bar\", \"userId\": 1}")
///   .build()?;
/// let response = request.send_with(&client).await?;
/// assert_eq!(response.status(), 201);
/// ```
///
#[inline]
pub fn post<T: IntoUrl>(url: T) -> Result<DeboaRequestBuilder> {
    DeboaRequest::post(url)
}

/// A utility function to create a QUERY request within DeboaRequest.
///
/// # Arguments
///
/// * `url` - The url to connect.
///
/// # Returns
///
/// * `Result<DeboaRequestBuilder>` - The request builder.
///
/// # Examples
///
/// ```rust,compile_fail
/// use deboa::{request::query};
/// use deboa_tokio::Client;
///
/// let client = Client::new();
///
/// let request = query("https://jsonplaceholder.typicode.com/posts")?
///   .raw_body(b"{\"title\": \"foo\", \"body\": \"bar\", \"userId\": 1}")
///   .build()?;
/// let response = request.send_with(&client).await?;
/// assert_eq!(response.status(), 201);
/// ```
///
#[inline]
pub fn query<T: IntoUrl>(url: T) -> Result<DeboaRequestBuilder> {
    DeboaRequest::query(url)
}

/// A utility function to create a PUT request within DeboaRequest.
///
/// # Arguments
///
/// * `url` - The url to connect.
///
/// # Returns
///
/// * `Result<DeboaRequestBuilder>` - The request builder.
///
/// # Examples
///
/// ```rust,compile_fail
/// use deboa::{request::put};
/// use deboa_tokio::Client;
///
/// let client = Client::new();
///
/// let request = put("https://jsonplaceholder.typicode.com/posts/1")?
///   .raw_body(b"{\"title\": \"foo\", \"body\": \"bar\", \"userId\": 1}")
///   .build()?;
/// let response = request.send_with(&client).await?;
/// assert_eq!(response.status(), 200);
/// ```
#[inline]
pub fn put<T: IntoUrl>(url: T) -> Result<DeboaRequestBuilder> {
    DeboaRequest::put(url)
}

/// A utility function to create a DELETE request within DeboaRequest.
///
/// # Arguments
///
/// * `url` - The url to connect.
///
/// # Returns
///
/// * `Result<DeboaRequestBuilder>` - The request builder.
///
/// # Examples
///
/// ```rust,compile_fail
/// use deboa::{request::delete};
/// use deboa_tokio::Client;
///
/// let client = Client::new();
///
/// let request = delete("https://jsonplaceholder.typicode.com/posts/1").build();
/// let response = request.send_with(&client).await?;
/// assert_eq!(response.status(), 200);
/// ```
#[inline]
pub fn delete<T: IntoUrl>(url: T) -> Result<DeboaRequestBuilder> {
    DeboaRequest::delete(url)
}

/// A utility function to create a PATCH request within DeboaRequest.
///
/// # Arguments
///
/// * `url` - The url to connect.
///
/// # Returns
///
/// * `Result<DeboaRequestBuilder>` - The request builder.
///
/// # Examples
///
/// ```rust,compile_fail
/// use deboa::{request::patch};
/// use deboa_tokio::Client;
///
/// let client = Client::new();
///
/// let request = patch("https://jsonplaceholder.typicode.com/posts/1")?
///   .raw_body(b"{\"title\": \"foo\"}")
///   .build()?;
/// let response = request.send_with(&client).await?;
/// assert_eq!(response.status(), 200);
/// ```
#[inline]
pub fn patch<T: IntoUrl>(url: T) -> Result<DeboaRequestBuilder> {
    DeboaRequest::patch(url)
}

/// A builder for constructing HTTP requests with various configurations.
///
/// `DeboaRequestBuilder` provides a fluent interface for building and customizing
/// HTTP requests. It supports setting headers, cookies, request bodies, and more.
///
/// # Examples
///
/// ```rust,ignore
/// use deboa::{request::post, Result};
/// use http::header;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///   let request = post("https://httpbin.org/post")?
///     .header(header::CONTENT_TYPE, "application/json")
///     .header(header::ACCEPT, "application/json")
///     .text(r#"{"key":"value"}"#)
///     .build()?;
///   Ok(())
/// }
/// ```
///
/// # Fields
///
/// * `url` - The target URL for the request
/// * `headers` - HTTP headers to include in the request
/// * `cookies` - Optional cookies to include in the request
/// * `method` - The HTTP method (GET, POST, etc.)
/// * `body` - The request body as raw bytes
/// * `form` - Optional form data for form submissions
pub struct DeboaRequestBuilder {
    inner: Request<HttpBody>,
}

impl DeboaRequestBuilder {
    /// Set the method of the request.
    ///
    /// # Arguments
    ///
    /// * `method` - The method.
    ///
    /// # Returns
    ///
    /// * `Self` - The request builder.
    ///
    #[inline]
    pub fn method(mut self, method: http::Method) -> Self {
        *self
            .inner
            .method_mut() = method;
        self
    }

    /// Set the protocol version for the request.
    ///
    /// # Arguments
    ///
    /// * `version` - The version.
    ///
    /// # Returns
    ///
    /// * `Self` - The request builder.
    ///
    pub fn version(mut self, version: http::Version) -> Self {
        *self
            .inner
            .version_mut() = version;
        self
    }

    /// Set the body of the request as raw bytes.
    ///
    /// # Arguments
    ///
    /// * `body` - The body.
    ///
    /// # Returns
    ///
    /// * `Self` - The request builder.
    ///
    #[inline]
    pub fn bytes(mut self, body: &[u8]) -> Self {
        *self
            .inner
            .body_mut() = HttpBody::from_bytes(body);
        self
    }

    /// Set the body of the request as raw bytes.
    ///
    /// # Arguments
    ///
    /// * `body` - The body.
    ///
    /// # Returns
    ///
    /// * `Self` - The request builder.
    ///
    #[inline]
    pub fn body(mut self, body: HttpBody) -> Self {
        *self
            .inner
            .body_mut() = body;
        self
    }

    /// Set the headers of the request.
    ///
    /// # Arguments
    ///
    /// * `headers` - The headers.
    ///
    /// # Returns
    ///
    /// * `Self` - The request builder.
    ///
    #[inline]
    pub fn headers<I>(mut self, headers: I) -> Self
    where
        I: IntoHeaders,
    {
        *self
            .inner
            .headers_mut() = headers
            .into_headers()
            .unwrap_or_default();
        self
    }

    /// Add a header to the request.
    ///
    /// # Arguments
    ///
    /// * `key` - The header key.
    /// * `value` - The header value.
    ///
    /// # Returns
    ///
    /// * `Self` - The request builder.
    ///
    /// # Examples
    ///
    /// ```rust,compile_fail
    /// use deboa::request::post;
    /// use http::header;
    ///
    /// let request = post("https://jsonplaceholder.typicode.com/posts")?
    ///   .header(header::CONTENT_TYPE, "application/json")
    ///   .raw_body(b"{\"title\": \"foo\", \"body\": \"bar\", \"userId\": 1}")
    ///   .build()?;
    /// let response = request.send_with(&mut client).await?;
    /// assert_eq!(response.status(), 201);
    /// ```
    ///
    #[inline]
    pub fn header(mut self, key: HeaderName, value: &str) -> Result<Self> {
        self.inner
            .headers_mut()
            .insert(
                key,
                HeaderValue::from_str(value)
                    .map_err(|e| DeboaError::Header { message: e.to_string() })?,
            );
        Ok(self)
    }

    /// Set the cookies of the request.
    ///
    /// # Arguments
    ///
    /// * `cookies` - The cookies.
    ///
    /// # Returns
    ///
    /// * `Self` - The request builder.
    ///
    #[inline]
    pub fn cookies(mut self, cookies: HashMap<String, DeboaCookie>) -> Result<Self> {
        self.inner
            .headers_mut()
            .insert(
                header::COOKIE,
                HeaderValue::from_str(
                    &cookies
                        .iter()
                        .map(|(k, v)| format!("{}={}", k, v.value()))
                        .collect::<Vec<_>>()
                        .join("; "),
                )
                .map_err(|e| DeboaError::Cookie { message: e.to_string() })?,
            );
        Ok(self)
    }

    /// Add a cookie to the request.
    ///
    /// # Arguments
    ///
    /// * `cookie` - The cookie.
    ///
    /// # Returns
    ///
    /// * `Self` - The request builder.
    ///
    #[inline]
    pub fn cookie(mut self, cookie: DeboaCookie) -> Result<Self> {
        if let Some(cookies) = self
            .inner
            .headers_mut()
            .get_mut(header::COOKIE)
        {
            let cookie_str = cookies
                .to_str()
                .map_err(|e| DeboaError::Cookie { message: e.to_string() })?;

            let cookie_str = format!("{}; {}={}", cookie_str, cookie.name(), cookie.value());
            *cookies = HeaderValue::from_str(&cookie_str)
                .map_err(|e| DeboaError::Cookie { message: e.to_string() })?;
        } else {
            self.inner
                .headers_mut()
                .insert(
                    header::COOKIE,
                    HeaderValue::from_str(&format!("{}={}", cookie.name(), cookie.value()))
                        .map_err(|e| DeboaError::Cookie { message: e.to_string() })?,
                );
        }
        Ok(self)
    }

    /// Set multipart form of the request.
    /// Content-Type will be set to `multipart/form-data` or `application/x-www-form-urlencoded`
    /// based on the enum variant.
    ///
    /// # Arguments
    ///
    /// * `form` - The form.
    ///
    /// # Returns
    ///
    /// * `Self` - The request builder.
    ///
    /// # Examples
    ///
    /// ```rust,compile_fail
    /// use deboa::request::post;
    /// use deboa::form::MultiPartForm;
    ///
    /// let mut form = MultiPartForm::builder();
    ///     .field("name", "deboa");
    ///     .field("version", "0.0.1");
    ///
    /// let request = post("https://jsonplaceholder.typicode.com/posts")?
    ///   .form(form.into())
    ///   .build()?;
    /// let response = request.send_with(&mut client).await?;
    /// assert_eq!(response.status(), 201);
    /// ```
    #[inline]
    pub fn form(mut self, form: Form) -> Result<Self> {
        let (content_type, body) = match form {
            Form::EncodedForm(form) => (form.content_type(), form.build()),
            Form::MultiPartForm(form) => (form.content_type(), form.build()),
        };
        let header_value = HeaderValue::from_str(&content_type)
            .map_err(|e| DeboaError::Header { message: e.to_string() })?;
        self.inner
            .headers_mut()
            .insert(header::CONTENT_TYPE, header_value);

        *self
            .inner
            .body_mut() = HttpBody::from_bytes(&body);
        Ok(self)
    }

    /// Set the body of the request as text.
    ///
    /// # Arguments
    ///
    /// * `text` - The text.
    ///
    /// # Returns
    ///
    /// * `Self` - The request builder.
    ///
    /// # Examples
    ///
    /// ```rust,compile_fail
    /// use deboa::request::post;
    ///
    /// let request = post("https://jsonplaceholder.typicode.com/posts")?
    ///   .header(header::CONTENT_TYPE, "application/json")
    ///   .text("text")
    ///   .build()?;
    /// let response = request.send_with(&mut client).await?;
    /// assert_eq!(response.status(), 201);
    /// ```
    #[inline]
    pub fn text(mut self, text: &str) -> Self {
        *self
            .inner
            .body_mut() = HttpBody::from_bytes(text.as_bytes());
        self
    }

    /// Set the body of the request as a type.
    ///
    /// # Arguments
    ///
    /// * `body_type` - The body type.
    /// * `body` - The body.
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - The request builder.
    ///
    /// # Examples
    ///
    /// ```rust,compile_fail
    /// use deboa::request::post;
    /// use deboa_extras::http::serde::JsonBody;
    ///
    /// let body = serde_json::json!({
    ///   "name": "deboa",
    ///   "version": "0.0.1"
    /// });
    ///
    /// let request = post("https://some.api.com/ping")?
    ///   .body_as(JsonBody, body)?;
    /// let response = request.send_with(&mut client).await?;
    /// assert_eq!(response.status(), 200);
    /// ```
    #[inline]
    pub fn body_as<T: RequestBody, B: Serialize>(self, body_type: T, body: B) -> Result<Self> {
        Ok(self
            .header(header::CONTENT_TYPE, body_type.mime_type())?
            .header(header::ACCEPT, body_type.mime_type())?
            .body(HttpBody::from_bytes(&body_type.serialize(body)?)))
    }

    /// Add bearer auth to the request.
    ///
    /// # Arguments
    ///
    /// * `token` - The token.
    ///
    /// # Returns
    ///
    /// * `Self` - The request builder.
    ///
    /// # Examples
    ///
    /// ```rust,compile_fail
    /// use deboa::request::post;
    ///
    /// let request = post("https://jsonplaceholder.typicode.com/posts")?
    ///   .header(header::CONTENT_TYPE, "application/json")
    ///   .bearer_auth("token")
    ///   .raw_body(b"{\"title\": \"foo\", \"body\": \"bar\", \"userId\": 1}")
    ///   .build()?;
    /// let response = request.send_with(&mut client).await?;
    /// assert_eq!(response.status(), 201);
    /// ```
    #[inline]
    pub fn bearer_auth(self, token: &str) -> Result<Self> {
        self.header(header::AUTHORIZATION, format!("Bearer {token}").as_str())
    }

    /// Add basic auth to the request.
    ///
    /// # Arguments
    ///
    /// * `username` - The username.
    /// * `password` - The password.
    ///
    /// # Returns
    ///
    /// * `Self` - The request builder.
    ///
    /// # Examples
    ///
    /// ```rust,compile_fail
    /// use deboa::request::post;
    ///
    /// let request = post("https://jsonplaceholder.typicode.com/posts")?
    ///   .header(header::CONTENT_TYPE, "application/json")
    ///   .basic_auth("username", "password")
    ///   .raw_body(b"{\"title\": \"foo\", \"body\": \"bar\", \"userId\": 1}")
    ///   .build()?;
    /// let response = request.send_with(&mut client).await?;
    /// assert_eq!(response.status(), 201);
    /// ```
    #[inline]
    pub fn basic_auth(self, username: &str, password: &str) -> Result<Self> {
        self.header(
            header::AUTHORIZATION,
            format!("Basic {}", STANDARD.encode(format!("{username}:{password}"))).as_str(),
        )
    }

    /// Build the request. Consuming the builder.
    ///
    /// # Returns
    ///
    /// * `Result<DeboaRequest>` - The request.
    ///
    /// # Panics
    ///
    /// * If an error occurs while building the request
    ///
    #[inline]
    pub fn build(self) -> Result<DeboaRequest> {
        Ok(DeboaRequest { inner: self.inner })
    }

    /// Send the request. Consuming the builder.
    ///
    /// # Arguments
    ///
    /// * `client` - The client to be used.
    ///
    /// # Returns
    ///
    /// * `Result<DeboaResponse>` - The response.
    ///
    /// # Examples
    ///
    /// ```rust,compile_fail
    /// use deboa::request::post;
    ///
    /// let request = post("https://jsonplaceholder.typicode.com/posts")?
    ///   .header(header::CONTENT_TYPE, "application/json")
    ///   .raw_body(b"{\"title\": \"foo\", \"body\": \"bar\", \"userId\": 1}")
    ///   .build()?;
    /// let response = request.send_with(&mut client).await?;
    /// assert_eq!(response.status(), 201);
    /// ```
    #[deprecated(note = "Use `send_with` method instead", since = "0.0.8")]
    #[inline]
    pub async fn go<T>(self, client: T) -> Result<DeboaResponse>
    where
        T: HttpClient,
    {
        client
            .execute(self.build()?)
            .await
    }

    /// Send the request. Consuming the builder.
    ///
    /// # Arguments
    ///
    /// * `client` - The client to be used.
    ///
    /// # Returns
    ///
    /// * `Result<DeboaResponse>` - The response.
    ///
    /// # Panics
    ///
    /// * If an error occurs while sending the request
    ///
    /// # Examples
    ///
    /// ```rust,compile_fail
    /// use deboa::request::post;
    ///
    /// let request = post("https://jsonplaceholder.typicode.com/posts")?
    ///   .header(header::CONTENT_TYPE, "application/json")
    ///   .raw_body(b"{\"title\": \"foo\", \"body\": \"bar\", \"userId\": 1}")
    ///   .build()?;
    /// let response = request.send_with(&mut client).await?;
    /// assert_eq!(response.status(), 201);
    /// ```
    #[inline]
    pub async fn send_with<T>(self, client: &T) -> Result<DeboaResponse>
    where
        T: HttpClient,
    {
        client
            .execute(self.build()?)
            .await
    }
}

/// Deboa request
pub struct DeboaRequest {
    inner: Request<HttpBody>,
}

impl Debug for DeboaRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeboaRequest")
            .field("url", &self.inner.uri())
            .field("headers", &self.inner.headers())
            .field("cookies", &None::<HashMap<String, DeboaCookie>>)
            .field("version", &self.inner.version())
            .field("method", &self.inner.method())
            .finish()
    }
}

/// Parse a string into a DeboaRequest.
///
/// # Arguments
///
/// * `s` - The string to parse.
///
/// # Returns
///
/// * `Result<DeboaRequest>` - The parsed request.
///
/// # Examples
///
/// ```rust,compile_fail
/// use deboa::request::DeboaRequest;
///
/// let request = DeboaRequest::from_str("GET https://jsonplaceholder.typicode.com/posts").unwrap();
/// assert_eq!(request.method(), http::Method::GET);
/// assert_eq!(request.url(), "https://jsonplaceholder.typicode.com/posts");
/// ```
impl FromStr for DeboaRequest {
    type Err = DeboaError;

    fn from_str(s: &str) -> Result<Self> {
        let lines = s.lines();

        let mut headers = HeaderMap::new();
        let mut url = String::new();
        let mut method = String::new();
        let mut body = Vec::new();
        let mut is_reading_body = false;

        let method_url_regex =
            Regex::new(r"(GET|POST|QUERY|PUT|DELETE|PATCH|HEAD|OPTIONS)\s+(https?://[^\s]+)");
        if let Err(e) = method_url_regex {
            error!("Failed to parse request: {}", e);
            return Err(DeboaError::Request(RequestError::Parse { message: e.to_string() }));
        }

        for line in lines {
            let line = line.trim();
            if !is_reading_body {
                let regex = method_url_regex
                    .as_ref()
                    .unwrap();
                let captures = regex.captures(line);
                if let Some(captures) = captures {
                    let method_cap = captures.get(1);
                    if method_cap.is_none() {
                        error!("Missing method in request format");
                        return Err(DeboaError::Request(RequestError::Parse {
                            message: "Missing method in request format".into(),
                        }));
                    }
                    let url_cap = captures.get(2);
                    if url_cap.is_none() {
                        error!("Missing url in request format");
                        return Err(DeboaError::Request(RequestError::Parse {
                            message: "Missing url in request format".into(),
                        }));
                    }
                    method = method_cap
                        .unwrap()
                        .as_str()
                        .to_string();
                    url = url_cap
                        .unwrap()
                        .as_str()
                        .to_string();
                    continue;
                }

                let header = line.split_once(':');
                if let Some(header) = header {
                    let header_name = HeaderName::from_bytes(
                        header
                            .0
                            .trim()
                            .as_bytes(),
                    )
                    .map_err(|_| {
                        error!("Invalid header name");
                        DeboaError::Request(RequestError::Parse {
                            message: "Invalid header name".into(),
                        })
                    })?;

                    let header_value = HeaderValue::from_bytes(
                        header
                            .1
                            .trim()
                            .as_bytes(),
                    )
                    .map_err(|_| {
                        error!("Invalid header value");
                        DeboaError::Request(RequestError::Parse {
                            message: "Invalid header value".into(),
                        })
                    })?;

                    headers.insert(header_name, header_value);
                    continue;
                }
            }

            if line.is_empty() && !url.is_empty() && !headers.is_empty() {
                is_reading_body = true;
                continue;
            }

            if is_reading_body {
                body.extend_from_slice(line.as_bytes());
            }
        }

        let uri = url
            .parse::<Uri>()
            .map_err(|e| {
                error!("Invalid URL: {}", e);
                DeboaError::Request(RequestError::Parse { message: "Invalid URL".into() })
            })?;

        if headers
            .get(header::HOST)
            .is_none()
        {
            if let Some(authority) = uri.authority() {
                headers.insert(header::HOST, HeaderValue::from_str(authority.as_str()).unwrap());
            }
        }

        let method = method
            .parse::<Method>()
            .map_err(|e| {
                error!("Invalid method: {}", e);
                DeboaError::Request(RequestError::Parse { message: "Invalid method".into() })
            })?;

        let mut builder = Request::builder()
            .uri(uri)
            .method(method)
            .version(Version::HTTP_2);

        *builder
            .headers_mut()
            .unwrap() = headers;

        let request = builder
            .body(HttpBody::from_bytes(&body))
            .map_err(|e| {
                error!("Failed to build request: {}", e);
                DeboaError::Request(RequestError::Parse {
                    message: "Failed to build request".into(),
                })
            })?;

        Ok(DeboaRequest { inner: request })
    }
}

impl AsRef<DeboaRequest> for DeboaRequest {
    fn as_ref(&self) -> &DeboaRequest {
        self
    }
}

impl AsMut<DeboaRequest> for DeboaRequest {
    fn as_mut(&mut self) -> &mut DeboaRequest {
        self
    }
}

impl DeboaRequest {
    /// Allow make a request.
    ///
    /// # Arguments
    ///
    /// * `url` - The url to be requested.
    /// * `method` - The method to be used.
    ///
    /// # Returns
    ///
    /// * `DeboaRequestBuilder` - The request builder.
    ///
    /// # Panics
    ///
    /// * If URL is invalid
    ///
    /// # Examples
    ///
    /// ``` compile_fail
    /// use deboa::request::post;
    ///
    /// let request = at("https://jsonplaceholder.typicode.com/posts", http::Method::POST)?
    ///   .header("Content-Type", "application/json")
    ///   .raw_body(b"{\"title\": \"foo\", \"body\": \"bar\", \"userId\": 1}")
    ///   .build()?;
    /// let response = request.send_with(&mut client).await?;
    /// assert_eq!(response.status(), 201);
    /// ```
    ///
    #[inline]
    pub fn at<T: IntoUrl>(url: T, method: http::Method) -> Result<DeboaRequestBuilder> {
        let parsed_url = url.into_url()?;

        let uri = parsed_url
            .to_string()
            .parse::<http::Uri>()
            .map_err(|e| DeboaError::Request(RequestError::UrlParse { message: e.to_string() }))?;

        let request = Request::builder()
            .method(method)
            .version(Version::HTTP_2)
            .header(header::HOST, uri.host().unwrap())
            .uri(uri)
            .body(HttpBody::from_bytes(&[]))
            .map_err(|e| DeboaError::Request(RequestError::Prepare { message: e.to_string() }))?;

        Ok(DeboaRequestBuilder { inner: request })
    }

    /// Allow make a GET request.
    ///
    /// # Arguments
    ///
    /// * `url` - The url to be requested.
    ///
    /// # Returns
    ///
    /// * `DeboaRequestBuilder` - The request builder.
    ///
    /// # Panics
    ///
    /// * If URL is invalid
    ///
    #[inline]
    pub fn from<T: IntoUrl>(url: T) -> Result<DeboaRequestBuilder> {
        DeboaRequest::at(url, Method::GET)
    }

    /// Allow make a POST request.
    ///
    /// # Arguments
    ///
    /// * `url` - The url to be requested.
    ///
    /// # Returns
    ///
    /// * `DeboaRequestBuilder` - The request builder.
    ///
    /// # Panics
    ///
    /// * If URL is invalid
    ///
    #[inline]
    pub fn to<T: IntoUrl>(url: T) -> Result<DeboaRequestBuilder> {
        DeboaRequest::at(url, Method::POST)
    }

    /// Allow make a GET request.
    ///
    /// # Arguments
    ///
    /// * `url` - The url to be requested.
    ///
    /// # Returns
    ///
    /// * `DeboaRequestBuilder` - The request builder.
    ///
    /// # Panics
    ///
    /// * If URL is invalid
    ///
    #[inline]
    pub fn get<T: IntoUrl>(url: T) -> Result<DeboaRequestBuilder> {
        Ok(DeboaRequest::from(url)?.method(Method::GET))
    }

    /// Allow make a POST request.
    ///
    /// # Arguments
    ///
    /// * `url` - The url to be requested.
    ///
    /// # Returns
    ///
    /// * `DeboaRequestBuilder` - The request builder.
    ///
    /// # Panics
    ///
    /// * If URL is invalid
    ///
    #[inline]
    pub fn post<T: IntoUrl>(url: T) -> Result<DeboaRequestBuilder> {
        Ok(DeboaRequest::to(url)?.method(Method::POST))
    }

    /// Allow make a QUERY request.
    ///
    /// # Arguments
    ///
    /// * `url` - The url to be requested.
    ///
    /// # Returns
    ///
    /// * `DeboaRequestBuilder` - The request builder.
    ///
    /// # Panics
    ///
    /// * If URL is invalid
    ///
    #[inline]
    pub fn query<T: IntoUrl>(url: T) -> Result<DeboaRequestBuilder> {
        Ok(DeboaRequest::to(url)?.method(Method::QUERY))
    }

    /// Allow make a PUT request.
    ///
    /// # Arguments
    ///
    /// * `url` - The url to be requested.
    ///
    /// # Returns
    ///
    /// * `DeboaRequestBuilder` - The request builder.
    ///
    /// # Panics
    ///
    /// * If URL is invalid
    ///
    #[inline]
    pub fn put<T: IntoUrl>(url: T) -> Result<DeboaRequestBuilder> {
        Ok(DeboaRequest::to(url)?.method(Method::PUT))
    }

    /// Allow make a PATCH request.
    ///
    /// # Arguments
    ///
    /// * `url` - The url to be requested.
    ///
    /// # Returns
    ///
    /// * `DeboaRequestBuilder` - The request builder.
    ///
    /// # Panics
    ///
    /// * If URL is invalid
    ///
    #[inline]
    pub fn patch<T: IntoUrl>(url: T) -> Result<DeboaRequestBuilder> {
        Ok(DeboaRequest::to(url)?.method(Method::PATCH))
    }

    /// Allow make a DELETE request.
    ///
    /// # Arguments
    ///
    /// * `url` - The url to be requested.
    ///
    /// # Returns
    ///
    /// * `DeboaRequestBuilder` - The request builder.
    ///
    /// # Panics
    ///
    /// * If URL is invalid
    ///
    #[inline]
    pub fn delete<T: IntoUrl>(url: T) -> Result<DeboaRequestBuilder> {
        Ok(DeboaRequest::from(url)?.method(Method::DELETE))
    }

    /// Create a request from parts and body.
    ///
    /// # Arguments
    ///
    /// * `parts` - The request parts.
    /// * `body` - The request body.
    ///
    /// # Returns
    ///
    /// * `DeboaRequest` - The request.
    ///
    /// # Errors
    ///
    /// * `DeboaError` - If the request is invalid.
    ///
    #[inline]
    pub fn from_parts(parts: http::request::Parts, body: HttpBody) -> Result<DeboaRequest> {
        let request = http::Request::from_parts(parts, body);
        Ok(DeboaRequest { inner: request })
    }

    /// Convert the request into parts and body.
    ///
    /// # Returns
    ///
    /// * `(http::request::Parts, HttpBody)` - The request parts and body.
    ///
    #[inline]
    pub fn into_parts(self) -> (http::request::Parts, HttpBody) {
        self.inner
            .into_parts()
    }

    /// Get request version at any time.
    ///
    /// # Returns
    ///
    /// * `http::Version` - The version.
    ///
    #[inline]
    pub fn version(&self) -> http::Version {
        self.inner.version()
    }

    /// Get request method at any time.
    ///
    /// # Returns
    ///
    /// * `http::Method` - The method.
    ///
    #[inline]
    pub fn method(&self) -> &http::Method {
        self.inner.method()
    }

    /// Allow get request url at any time.
    ///
    /// # Returns
    ///
    /// * `Url` - The url.
    ///
    #[inline]
    pub fn uri(&self) -> &Uri {
        self.inner.uri()
    }

    /// Allow get request headers at any time.
    ///
    /// # Returns
    ///
    /// * `HeaderMap` - The headers.
    ///
    #[inline]
    pub fn headers(&self) -> &HeaderMap {
        self.inner.headers()
    }

    /// Return mutable headers
    ///
    /// # Returns
    ///
    /// * `&mut HeaderMap` - The headers.
    ///
    #[inline]
    pub fn headers_mut(&mut self) -> &mut HeaderMap {
        self.inner
            .headers_mut()
    }

    /// Allow get cookies at any time.
    ///
    /// # Returns
    ///
    /// * `Option<&HashMap<String, DeboaCookie>>` - The cookies.
    ///
    #[inline]
    pub fn cookies(&self) -> Option<HashMap<String, DeboaCookie>> {
        self.inner
            .headers()
            .get("cookie")
            .map(|cookie| {
                // Parse cookies from header
                let mut cookies = HashMap::new();
                if let Ok(cookie_str) = cookie.to_str() {
                    for cookie_pair in cookie_str.split(';') {
                        let trimmed = cookie_pair.trim();
                        if let Some((name, value)) = trimmed.split_once('=') {
                            cookies.insert(name.to_string(), DeboaCookie::new(name, value));
                        }
                    }
                }
                cookies
            })
    }

    /// Allow get body at any time.
    ///
    /// # Returns
    ///
    /// * `&DeboaBody` - The body.
    ///
    pub fn body(self) -> Request<HttpBody> {
        self.inner
    }
}
mod private {
    pub trait IntoRequestSealed {}
    pub trait IntoHeadersSealed {}
    pub trait MethodExtSealed {}
}

impl private::IntoRequestSealed for DeboaRequest {}

impl private::IntoRequestSealed for &str {}

impl private::IntoRequestSealed for String {}

impl private::IntoRequestSealed for Url {}

impl private::IntoHeadersSealed for HeaderMap {}

impl private::IntoHeadersSealed for Vec<(HeaderName, String)> {}

impl private::IntoHeadersSealed for Vec<(String, String)> {}

impl<'a> private::IntoHeadersSealed for Vec<(&'a str, &'a str)> {}

impl private::MethodExtSealed for Method {}

impl private::MethodExtSealed for &str {}
