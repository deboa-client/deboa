# deboa

[![Crates.io downloads](https://img.shields.io/crates/d/deboa)](https://crates.io/crates/deboa) [![crates.io](https://img.shields.io/crates/v/deboa?style=flat-square)](https://crates.io/crates/deboa) [![Build Status](https://github.com/deboa-client/deboa/actions/workflows/rust.yml/badge.svg?event=push)](https://github.com/deboa-client/deboa/actions/workflows/rust.yml) ![Crates.io MSRV](https://img.shields.io/crates/msrv/deboa) [![Documentation](https://docs.rs/deboa/badge.svg)](https://docs.rs/deboa/latest/deboa) [![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/deboa-client/deboa/blob/main/LICENSE.md)  [![codecov](https://codecov.io/gh/deboa-client/deboa/graph/badge.svg?token=T0HSBAPVSI)](https://codecov.io/gh/deboa-client/deboa)

## Description

**deboa** ("fine" portuguese slang) is a straightforward, non opinionated, developer-centric HTTP client library for Rust. It offers a rich array of modern features—from flexible authentication and serialization formats to runtime compatibility and middleware support—while maintaining simplicity and ease of use. It’s especially well-suited for Rust projects that require a lightweight, efficient HTTP client without sacrificing control or extensibility.

## Attention

This release has a major api change. Please check the [migration guide](https://github.com/deboa-client/deboa/blob/main/MIGRATION_GUIDE.md) for more information.

## Features

- easily add, remove and update headers
- helpers to add basic and bearer auth
- set retries and timeout
- compression (gzip, deflate, br)
- pluggable hooks (interceptors)
- pluggable serialization (json, xml, msgpack)
- cookies support
- urlencoded and multipart forms
- comprehensive error handling
- response streaming
- upgrade support (websocket, etc.)
- runtime compatibility (tokio, smol and compio)
- http 1/2/3 support via runtime crates

## Benchmark Results

As of the latest benchmark run, Deboa demonstrates competitive performance compared to Reqwest.

### Get Request

|            | `Deboa`                  | `Reqwest`                        |
|:-----------|:-------------------------|:-------------------------------- |
| **`100`**  | `46.37 ms` (✅ **1.00x**) | `48.67 ms` (✅ **1.05x slower**)  |
| **`500`**  | `46.47 ms` (✅ **1.00x**) | `47.32 ms` (✅ **1.02x slower**)  |
| **`1000`** | `46.36 ms` (✅ **1.00x**) | `47.34 ms` (✅ **1.02x slower**)  |

## Install

Either run from command line:

`cargo add deboa http`

Or add to your `Cargo.toml`:

```toml
deboa = { version = "0.0.9" }
deboa-extras = { version = "0.1.0" }
deboa-tokio = { version = "0.1.0" }
http = "1.3.1"
```

## Crate features

- http1
- http2 (default)
- http3
- rust-tls
- native-tls

## Usage

```rust, compile_fail
use deboa::{
    request::{DeboaRequest, FetchWith, get},
    Result
};
use deboa_extras::http::{self, serde::json::JsonBody};
use deboa_tokio::Client;

use ::http::Method;

#[derive(Debug, serde::Deserialize)]
pub struct Post {
    pub id: u64,
    pub title: String,
    pub body: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::new();

    /*

    // You can also use the Fetch trait to issue requests

    let posts: Vec<Post> = "https://jsonplaceholder.typicode.com/posts"
      .fetch_with(client)
      .await?
      .body_as(JsonBody)
      .await?;

    // or use at, from (defaults to GET) and to (defaults to POST) methods:

    let posts: Vec<Post> = DeboaRequest::at("https://jsonplaceholder.typicode.com/posts", Method::GET)?
      .send_with(client)
      .await?
      .body_as(JsonBody)
      .await?;

    // shifleft? Yes sir! Defaults to GET, but you can change it, same for headers.

    let request = &client << "https://jsonplaceholder.typicode.com/posts";
    let posts: Vec<Post> = client.execute(request)
      .await?
      .body_as(JsonBody)
      .await?;

    // or simply:

    let posts: Vec<Post> = client
      .execute("https://jsonplaceholder.typicode.com/posts")
      .await?
      .body_as(JsonBody)
      .await?;

    // you can also post a json body

    let body = serde_json::json!({
      "id": 100,
      "title": "Some title",
      "body": "Some body"
    });

    let request = post("https://jsonplaceholder.typicode.com/posts")?
      .header(header::CONTENT_TYPE, "application/json")
      .body_as(JsonBody, body)?;
    let response = request.send_with(&mut client).await?;
    assert_eq!(response.status(), 201);

    */

    let posts: Vec<Post> = get("https://jsonplaceholder.typicode.com/posts")?
      .send_with(client)
      .await?
      .body_as(JsonBody)
      .await?;

    println!("posts: {:#?}", posts);

    Ok(())
}
```

## Create project from template

You can create a new project from the template using `cargo generate`:

`cargo generate deboa-client/deboa-templates`

## License

MIT or Apache-2.0

## Author

Rogerio Pereira Araujo <rogerio.araujo@gmail.com>
