#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TS2};
use quote::quote;
use syn::parse_macro_input;

mod actions;

#[proc_macro]
/// Make a GET request to the specified URL.
///
/// The `get!` macro is used to make a GET request to the specified URL.
///
/// You can use the `JsonBody`, `XmlBody`, `MsgPack` type for JSON, XML
/// and MessagePack serialization.
///
/// get!(
///     url=> url,
///     headers => vec![("User-Agent", "deboa")],
///     client => &client,
///     res_body_ty => JsonBody,
///     res_ty => Article
/// )
///
/// # Arguments
///
/// * `url`         - URL to make the GET request to.
/// * `client`      - Client variable to use for the request.
/// * `res_body_ty` - Body type of the response.
/// * `res_ty`      - Type of the response.
///
/// Please note url can be a string literal or a variable.
///
/// # Example
///
/// ```rust, no_run, compile_fail
/// use deboa_tokio::Client;
/// use deboa_macros::get;
/// use deboa_extras::serde::json::JsonBody;
///
/// #[derive(serde::Deserialize)]
/// struct Post {
///     id: u32,
///     title: String,
///     body: String,
///     userId: u32,
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut client = Client::default();
///     let response = get!(
///         url => "https://jsonplaceholder.typicode.com/posts",
///         client => &client,
///         res_body_ty => JsonBody,
///         res_ty => Vec<Post>
///     );
///     assert_eq!(response.len(), 100);
///     Ok(())
/// }
/// ```
pub fn get(item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(item as actions::get::GetArgs);

    let url = match args.url {
        Some(e) => e,
        None => {
            return syn::Error::new(Span::call_site(), "Missing required field: 'url'")
                .to_compile_error()
                .into()
        }
    };
    let client = match args.client {
        Some(e) => e,
        None => {
            return syn::Error::new(Span::call_site(), "Missing required field: 'client'")
                .to_compile_error()
                .into()
        }
    };

    let headers = match args.headers {
        Some(headers) => quote! { .headers(#headers) },
        None => TS2::new(),
    };

    let body = match (args.res_body_ty, args.res_ty) {
        (Some(res_body_ty), Some(res_ty)) => {
            quote! { .body_as::<#res_body_ty, #res_ty>(#res_body_ty).await? }
        }
        (Some(_), None) => {
            return syn::Error::new(Span::call_site(), "Missing required field: 'res_ty'")
                .to_compile_error()
                .into()
        }
        _ => TS2::new(),
    };

    let expanded = quote! {
        deboa::HttpClient::execute(
            #client,
            deboa::request::DeboaRequest::get(#url)?
                #headers
                .build()?,
        )
        .await?
        #body
    };

    TokenStream::from(expanded)
}

#[proc_macro]
/// Make a QUERY request to the specified URL.
///
/// The `query!` macro is used to make a QUERY request to the specified URL.
///
/// query!(
///     data => data,
///     req_body_ty => XmlBody,
///     url => "http://example.com",
///     client => &client
/// );
///
/// # Arguments
///
/// * `input`       - The input to send with the request.
/// * `req_body_ty` - The body serialization format of the request.
/// * `url`         - The URL to make the QUERY request to.
/// * `headers`     - The headers to send with the request.
/// * `client`      - The client variable to use for the request.
/// * `res_body_ty` - The body serialization format of the response.
/// * `res_ty`      - The type of the response.
///
/// Please note url can be a string literal or a variable.
///
/// # Example
///
/// ## Without response body deserialization
///
/// ```rust, no_run, compile_fail
/// use deboa_macros::query;
/// use deboa_extras::serde::json::JsonBody;
/// use deboa_tokio::TokioClient;
///
/// #[derive(serde::Serialize)]
/// struct Search {
///     title: String,
///     body: String,
///     userId: u32,
/// }
///
/// #[derive(serde::Deserialize)]
/// struct Post {
///     id: u32,
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = TokioClient::default();
///     let data = Search {
///         title: "foo".to_string(),
///         body: "bar".to_string(),
///         userId: 1,
///     };
///     let response = query!(
///         data => data,
///         req_body_ty => JsonBody,
///         url => "https://jsonplaceholder.typicode.com/posts",
///         headers => vec!(("Content-Type", "application/json")),
///         client => &client
///     );
///     Ok(())
/// }
/// ```
///
/// ## With response body deserialization
///
/// ```rust, no_run, compile_fail
/// use deboa_macros::post;
/// use deboa_extras::serde::json::JsonBody;
/// use deboa_tokio::TokioClient;
///
/// #[derive(serde::Serialize)]
/// struct Post {
///     title: String,
///     body: String,
///     userId: u32,
/// }
///
/// #[derive(serde::Deserialize)]
/// struct CreatedPost {
///     id: u32,
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = TokioClient::default();
///     let data = Post {
///         title: "foo".to_string(),
///         body: "bar".to_string(),
///         userId: 1,
/// };
///     let response = post!(
///         data => data,
///         req_body_ty => JsonBody,
///         url => "https://jsonplaceholder.typicode.com/posts",
///         client => &client,
///         res_body_ty => JsonBody,
///         res_ty => CreatedPost
///     );
///     assert_eq!(response.id, 1);
///     Ok(())
/// }
/// ```
pub fn query(item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(item as actions::query::QueryArgs);

    let url = match args.url {
        Some(e) => e,
        None => {
            return syn::Error::new(Span::call_site(), "Missing required field: 'url'")
                .to_compile_error()
                .into()
        }
    };
    let client = match args.client {
        Some(e) => e,
        None => {
            return syn::Error::new(Span::call_site(), "Missing required field: 'client'")
                .to_compile_error()
                .into()
        }
    };

    let headers = match args.headers {
        Some(headers) => quote! { .headers(#headers) },
        None => TS2::new(),
    };

    let req_body = match (args.req_body_ty, args.data) {
        (Some(req_body_ty), Some(data)) => quote! { .body_as(#req_body_ty, #data)? },
        (Some(_), None) => {
            return syn::Error::new(Span::call_site(), "Missing required field: 'data'")
                .to_compile_error()
                .into()
        }
        _ => TS2::new(),
    };

    let res_body = match (args.res_body_ty, args.res_ty) {
        (Some(res_body_ty), Some(res_ty)) => {
            quote! { .body_as::<#res_body_ty, #res_ty>(#res_body_ty).await? }
        }
        (Some(_), None) => {
            return syn::Error::new(Span::call_site(), "Missing required field: 'res_ty'")
                .to_compile_error()
                .into()
        }
        _ => TS2::new(),
    };

    // Generate the final token stream
    let expanded = quote! {
        deboa::HttpClient::execute(
            #client,
            deboa::request::DeboaRequest::query(#url)?
                #headers
                #req_body
                .build()?,
        )
        .await?
        #res_body
    };

    TokenStream::from(expanded)
}

#[proc_macro]
/// Make a POST request to the specified URL.
///
/// The `post!` macro is used to make a POST request to the specified URL.
///
/// post!(
///     data => data,
///     req_body_ty => JsonBody,
///     url => "http://example.com",
///     headers => headers,
///     client => &client,
///     res_body_ty => JsonBody,
///     res_ty => Post
/// );
///
/// # Arguments
///
/// * `data`        - Data to send with the request.
/// * `req_body_ty` - Body serialization format of the request.
/// * `url`         - URL to make the POST request to.
/// * `headers`     - Headers to send with the request.
/// * `client`      - Client variable to use for the request.
/// * `res_body_ty` - Body serialization format of the response.
/// * `res_ty`      - Type of the response.
///
/// Please note url can be a string literal or a variable.
///
/// # Example
///
/// ## Without response body deserialization
///
/// ```rust, no_run, compile_fail
/// use deboa_macros::post;
/// use deboa_extras::serde::json::JsonBody;
/// use deboa_tokio::TokioClient;
///
/// #[derive(serde::Serialize)]
/// struct Post {
///     title: String,
///     body: String,
///     userId: u32,
/// }
///
/// #[derive(serde::Deserialize)]
/// struct CreatedPost {
///     id: u32,
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = TokioClient::default();
///     let data = Post {
///         title: "foo".to_string(),
///         body: "bar".to_string(),
///         userId: 1,
///     };
///     let response = post!(
///         data => data,
///         req_body_ty => JsonBody,
///         url => "https://jsonplaceholder.typicode.com/posts",
///         headers => vec!(("Content-Type", "application/json")),
///         client => &client
///     );
///     Ok(())
/// }
/// ```
///
/// ## With response body deserialization
///
/// ```rust, no_run, compile_fail
/// use deboa_macros::post;
/// use deboa_extras::serde::json::JsonBody;
/// use deboa_tokio::TokioClient;
///
/// #[derive(serde::Serialize)]
/// struct Post {
///     title: String,
///     body: String,
///     userId: u32,
/// }
///
/// #[derive(serde::Deserialize)]
/// struct CreatedPost {
///     id: u32,
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = TokioClient::default();
///     let data = Post {
///         title: "foo".to_string(),
///         body: "bar".to_string(),
///         userId: 1,
/// };
///     let response = post!(
///         data => data,
///         req_body_ty => JsonBody,
///         url => "https://jsonplaceholder.typicode.com/posts",
///         client => &client,
///         res_body_ty => JsonBody,
///         res_ty => CreatedPost
///     );
///     assert_eq!(response.id, 1);
///     Ok(())
/// }
/// ```
pub fn post(item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(item as actions::post::PostArgs);

    let url = match args.url {
        Some(e) => e,
        None => {
            return syn::Error::new(Span::call_site(), "Missing required field: 'url'")
                .to_compile_error()
                .into()
        }
    };
    let client = match args.client {
        Some(e) => e,
        None => {
            return syn::Error::new(Span::call_site(), "Missing required field: 'client'")
                .to_compile_error()
                .into()
        }
    };

    let headers = match args.headers {
        Some(headers) => quote! { .headers(#headers) },
        None => TS2::new(),
    };

    let req_body = match (args.req_body_ty, args.data) {
        (Some(req_body_ty), Some(data)) => quote! { .body_as(#req_body_ty, #data)? },
        (Some(_), None) => {
            return syn::Error::new(Span::call_site(), "Missing required field: 'data'")
                .to_compile_error()
                .into()
        }
        _ => TS2::new(),
    };

    let res_body = match (args.res_body_ty, args.res_ty) {
        (Some(res_body_ty), Some(res_ty)) => {
            quote! { .body_as::<#res_body_ty, #res_ty>(#res_body_ty).await? }
        }
        (Some(_), None) => {
            return syn::Error::new(Span::call_site(), "Missing required field: 'res_ty'")
                .to_compile_error()
                .into()
        }
        _ => TS2::new(),
    };

    // Generate the final token stream
    let expanded = quote! {
        deboa::HttpClient::execute(
            #client,
            deboa::request::DeboaRequest::post(#url)?
                #headers
                #req_body
                .build()?,
        )
        .await?
        #res_body
    };

    TokenStream::from(expanded)
}

#[proc_macro]
/// Make a PUT request to the specified URL.
///
/// put!(
///     data => data,
///     req_body_ty => JsonBody,
///     url => "http://example.com",
///     client => &mut client
/// );
///
/// # Arguments
///
/// * `data`        - Data to send with request.
/// * `req_body_ty` - Body serialization format of request.
/// * `url`         - URL to make the PUT request to.
/// * `headers`     - Headers to send with request.
/// * `client`      - Client variable to use for request.
///
/// Please note url can be a string literal or a variable.
///
/// # Example
///
/// ```rust, no_run, compile_fail
/// use deboa_extras::serde::json::JsonBody;
/// use deboa_macros::put;
/// use deboa_tokio::TokioClient;
///
/// #[derive(serde::Serialize)]
/// struct Post {
///     title: String,
///     body: String,
///     userId: u32,
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = TokioClient::default();
///     let data = Post {
///         title: "foo".to_string(),
///         body: "bar".to_string(),
///         userId: 1,
///     };
///     put!(
///         data => data,
///         req_body_ty => JsonBody,
///         url => "https://jsonplaceholder.typicode.com/posts/1",
///         client => &client
///     );
///     Ok(())
/// }
/// ```
pub fn put(item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(item as actions::put::PutArgs);

    let url = match args.url {
        Some(e) => e,
        None => {
            return syn::Error::new(Span::call_site(), "Missing required field: 'url'")
                .to_compile_error()
                .into()
        }
    };
    let client = match args.client {
        Some(e) => e,
        None => {
            return syn::Error::new(Span::call_site(), "Missing required field: 'client'")
                .to_compile_error()
                .into()
        }
    };

    let headers = match args.headers {
        Some(headers) => quote! { .headers(#headers) },
        None => TS2::new(),
    };

    let req_body = match (args.req_body_ty, args.data) {
        (Some(req_body_ty), Some(data)) => quote! { .body_as(#req_body_ty, #data)? },
        (Some(_), None) => {
            return syn::Error::new(Span::call_site(), "Missing required field: 'data'")
                .to_compile_error()
                .into()
        }
        _ => TS2::new(),
    };

    let res_body = match (args.res_body_ty, args.res_ty) {
        (Some(res_body_ty), Some(res_ty)) => {
            quote! { .body_as::<#res_body_ty, #res_ty>(#res_body_ty).await? }
        }
        (Some(_), None) => {
            return syn::Error::new(Span::call_site(), "Missing required field: 'res_ty'")
                .to_compile_error()
                .into()
        }
        _ => TS2::new(),
    };

    // Generate the final token stream
    let expanded = quote! {
        deboa::HttpClient::execute(
            #client,
            deboa::request::DeboaRequest::put(#url)?
                #headers
                #req_body
                .build()?,
        )
        .await?
        #res_body
    };

    TokenStream::from(expanded)
}

#[proc_macro]
/// Make a PATCH request to the specified URL.
///
/// patch!(
///     data => data,
///     req_body_ty => JsonBody,
///     url => "http://example.com",
///     client => &mut client
/// );
///
/// # Arguments
///
/// * `data`        - Data to send with request.
/// * `req_body_ty` - Body serialization format of request.
/// * `url`         - URL to make the PATCH request to.
/// * `headers`     - Headers to send with request.
/// * `client`      - Client variable to use for request.
///
/// Please note url can be a string literal or a variable.
///
/// # Example
///
/// ```rust, no_run, compile_fail
/// use deboa_extras::serde::json::JsonBody;
/// use deboa_macros::patch;
/// use deboa_tokio::TokioClient;
///
/// #[derive(serde::Serialize)]
/// struct Post {
///     title: String,
///     body: String,
///     userId: u32,
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = Client::default();
///     let data = Post {
///         title: "foo".to_string(),
///         body: "bar".to_string(),
///         userId: 1,
///     };
///     patch!(
///         data => data,
///         req_body_ty => JsonBody,
///         url => "https://jsonplaceholder.typicode.com/posts/1",
///         client => &client
///     );
///     Ok(())
/// }
/// ```
pub fn patch(item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(item as actions::patch::PatchArgs);

    let url = match args.url {
        Some(e) => e,
        None => {
            return syn::Error::new(Span::call_site(), "Missing required field: 'url'")
                .to_compile_error()
                .into()
        }
    };
    let client = match args.client {
        Some(e) => e,
        None => {
            return syn::Error::new(Span::call_site(), "Missing required field: 'client'")
                .to_compile_error()
                .into()
        }
    };

    let headers = match args.headers {
        Some(headers) => quote! { .headers(#headers) },
        None => TS2::new(),
    };

    let req_body = match (args.req_body_ty, args.data) {
        (Some(req_body_ty), Some(data)) => quote! { .body_as(#req_body_ty, #data)? },
        (Some(_), None) => {
            return syn::Error::new(Span::call_site(), "Missing required field: 'data'")
                .to_compile_error()
                .into()
        }
        _ => TS2::new(),
    };

    let res_body = match (args.res_body_ty, args.res_ty) {
        (Some(res_body_ty), Some(res_ty)) => {
            quote! { .body_as::<#res_body_ty, #res_ty>(#res_body_ty).await? }
        }
        (Some(_), None) => {
            return syn::Error::new(Span::call_site(), "Missing required field: 'res_ty'")
                .to_compile_error()
                .into()
        }
        _ => TS2::new(),
    };

    // Generate the final token stream
    let expanded = quote! {
        deboa::HttpClient::execute(
            #client,
            deboa::request::DeboaRequest::patch(#url)?
                #headers
                #req_body
                .build()?,
        )
        .await?
        #res_body
    };

    TokenStream::from(expanded)
}

#[proc_macro]
/// Make a DELETE request to the specified URL.
///
/// delete!(
///     url => "http://example.com",
///     client => &mut client
/// );
///
/// # Arguments
///
/// * `url`     - URL to make the DELETE request to.
/// * `headers` - Headers to send with request.
/// * `client`  - Client variable to use for request.
///
/// Please note url can be a string literal or a variable.
///
/// # Example
///
/// ```rust, no_run, compile_fail
/// use deboa_macros::delete;
/// use deboa_tokio::TokioClient;
///
/// #[derive(serde::Deserialize)]
/// struct Post {
///     id: u32,
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = TokioClient::default();
///     let response = delete!(url => "https://jsonplaceholder.typicode.com/posts/1", client => &client);
///     Ok(())
/// }
/// ```
pub fn delete(item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(item as actions::delete::DeleteArgs);

    // Enforce required fields
    let url = match args.url {
        Some(e) => e,
        None => {
            return syn::Error::new(Span::call_site(), "Missing required field: 'url'")
                .to_compile_error()
                .into()
        }
    };
    let client = match args.client {
        Some(e) => e,
        None => {
            return syn::Error::new(Span::call_site(), "Missing required field: 'client'")
                .to_compile_error()
                .into()
        }
    };

    let headers = match args.headers {
        Some(headers) => quote! { .headers(#headers) },
        None => TS2::new(),
    };

    // Generate the final token stream
    let expanded = quote! {
        deboa::HttpClient::execute(
            #client,
            deboa::request::DeboaRequest::delete(#url)?
                #headers
                .build()?,
        )
        .await?
    };

    TokenStream::from(expanded)
}

#[proc_macro]
/// Make a GET request to the specified URL.
///
/// The `fetch!` macro is a more generic version of the `get!` macro.
///
/// You can use the `JsonBody`, `XmlBody`, `MsgPack` type for JSON, XML
/// and MessagePack serialization.
///
/// fetch!(
///     url => "http://example.com",
///     client => &mut client,
///     res_body_ty => JsonBody,
///     res_ty => User
/// );
///
/// # Arguments
///
/// * `url`         - URL to make the GET request to.
/// * `headers`     - Headers to send with request.
/// * `client`      - Client variable to use request.
/// * `res_body_ty` - Body serialization format of response.
/// * `res_ty`      - Type of response.
///
/// Please note url can be a string literal or a variable.
///
/// # Example
///
/// ```rust, no_run, compile_fail
/// use deboa_extras::serde::json::JsonBody;
/// use deboa_macros::fetch;
/// use deboa_tokio::TokioClient;
///
/// #[derive(serde::Deserialize)]
/// struct Post {
///     id: u32,
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = TokioClient::default();
///     let response = fetch!(
///         url => "https://jsonplaceholder.typicode.com/posts",
///         client => &client,
///         res_body_ty => JsonBody,
///         res_ty => Post
///     );
///     assert_eq!(response.id, 1);
///     Ok(())
/// }
/// ```
pub fn fetch(item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(item as actions::get::GetArgs);

    // Enforce required fields
    let url = match args.url {
        Some(e) => e,
        None => {
            return syn::Error::new(Span::call_site(), "Missing required field: 'url'")
                .to_compile_error()
                .into()
        }
    };
    let client = match args.client {
        Some(e) => e,
        None => {
            return syn::Error::new(Span::call_site(), "Missing required field: 'client'")
                .to_compile_error()
                .into()
        }
    };

    let headers = match args.headers {
        Some(headers) => quote! { .headers(#headers) },
        None => TS2::new(),
    };

    let body = match (args.res_body_ty, args.res_ty) {
        (Some(res_body_ty), Some(res_ty)) => {
            quote! { .body_as::<#res_body_ty, #res_ty>(#res_body_ty).await? }
        }
        (Some(_), None) => {
            return syn::Error::new(Span::call_site(), "Missing required field: 'res_ty'")
                .to_compile_error()
                .into()
        }
        _ => TS2::new(),
    };

    // Generate the final token stream
    let expanded = quote! {
        deboa::HttpClient::execute(
            #client,
            deboa::request::DeboaRequest::get(#url)?
                #headers
                .build()?,
        )
        .await?
        #body
    };

    TokenStream::from(expanded)
}

#[proc_macro]
/// Submit a request to the specified URL.
///
/// The `submit!` macro is a more generic version of the `post!` macro.
///
/// subtmi!(
///     method => Method::POST,
///     url => "http://example.com",
///     client => &mut client,
///     data => "something"
/// )
///
/// # Arguments
///
/// * `method`      - HTTP method to use.
/// * `data`        - Data to send with request.
/// * `url`         - URL to make the GET request to.
/// * `headers`     - Headers to send with request.
/// * `client`      - Client variable to use for request.
///
/// Please note url can be a string literal or a variable.
///
/// # Example
///
/// ```rust, no_run, compile_fail
/// use deboa_macros::submit;
/// use deboa_tokio::TokioClient;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = TokioClient::default();
///     submit!(
///         method => http::Method::POST,
///         data => "user=deboa",
///         url => "https://jsonplaceholder.typicode.com/posts",
///         client => &client
///     );
///     Ok(())
/// }
/// ```
pub fn submit(item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(item as actions::submit::SubmitArgs);

    // Enforce required fields
    let url = match args.url {
        Some(e) => e,
        None => {
            return syn::Error::new(Span::call_site(), "Missing required field: 'url'")
                .to_compile_error()
                .into()
        }
    };
    let client = match args.client {
        Some(e) => e,
        None => {
            return syn::Error::new(Span::call_site(), "Missing required field: 'client'")
                .to_compile_error()
                .into()
        }
    };
    let method = match args.method {
        Some(e) => e,
        None => {
            return syn::Error::new(Span::call_site(), "Missing required field: 'method'")
                .to_compile_error()
                .into()
        }
    };

    let headers = match args.headers {
        Some(headers) => quote! { .headers(#headers) },
        None => TS2::new(),
    };

    let data = match args.data {
        Some(data) => quote! { .text(#data) },
        None => TS2::new(),
    };

    // Generate the final token stream
    let expanded = quote! {
        deboa::HttpClient::execute(
            #client,
            deboa::request::DeboaRequest::at(#url, #method)?
                #headers
                #data
                .build()?,
        )
        .await?
    };

    TokenStream::from(expanded)
}

#[proc_macro]
/// Make a GET request to the specified URL, returning a stream.
///
/// The `stream!` macro is used to make a GET request to the specified URL
///
/// stream!(
///     url => "http://example.com",
///     client => &mut client
/// );
///
/// # Arguments
///
/// * `url`     - URL to make the GET request to.
/// * `headers` - Headers to send with request.
/// * `client`  - Client variable to use for request.
///
/// Please note url can be a string literal or a variable.
///
/// # Example
///
/// ```rust, no_run, compile_fail
/// use deboa_macros::stream;
/// use deboa_tokio::TokioClient;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = TokioClient::default();
///     let response = stream!(url => "https://jsonplaceholder.typicode.com/posts", client => &client);
///     Ok(())
/// }
/// ```
pub fn stream(item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(item as actions::get::GetArgs);

    // Enforce required fields
    let url = match args.url {
        Some(e) => e,
        None => {
            return syn::Error::new(Span::call_site(), "Missing required field: 'url'")
                .to_compile_error()
                .into()
        }
    };
    let client = match args.client {
        Some(e) => e,
        None => {
            return syn::Error::new(Span::call_site(), "Missing required field: 'client'")
                .to_compile_error()
                .into()
        }
    };

    let headers = match args.headers {
        Some(headers) => quote! { .headers(#headers) },
        None => TS2::new(),
    };

    // Generate the final token stream
    let expanded = quote! {
        deboa::HttpClient::execute(
            #client,
            deboa::request::DeboaRequest::get(#url)?
                #headers
                .build()?,
        )
        .await?
        .stream()
    };

    TokenStream::from(expanded)
}
