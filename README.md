# Deboa

[![crates.io](https://img.shields.io/crates/v/deboa?style=flat-square)](https://crates.io/crates/deboa) [![Build Status](https://github.com/deboa-client/deboa/actions/workflows/rust.yml/badge.svg?event=push)](https://github.com/deboa-client/deboa/actions/workflows/rust.yml) [![codecov](https://codecov.io/gh/deboa-client/deboa/graph/badge.svg?token=T0HSBAPVSI)](https://codecov.io/gh/deboa-client/deboa) [![Documentation](https://docs.rs/deboa/badge.svg)](https://docs.rs/deboa/latest/deboa)

## Description

**deboa** ("fine" portuguese slang) is a straightforward, non opinionated, developer-centric HTTP client library for Rust. It offers a rich array of modern features—from flexible authentication and serialization formats to runtime compatibility and middleware support—while maintaining simplicity and ease of use. It’s especially well-suited for Rust projects that require a lightweight, efficient HTTP client without sacrificing control or extensibility.

Built using [hyper](https://github.com/hyperium/hyper).

## Attention

This release has a major api change. Please check the [migration guide](https://github.com/deboa-client/deboa/blob/main/MIGRATION_GUIDE.md) for more information. Keep in mind API for prior to 0.1.0 is subject to change. Proper deprecation will be added in the next stable release.

## Install

```toml
deboa = { version = "0.1.3" }
```

## Runtimes

- [compio](https://github.com/compio-rs/compio)
- [glommio](https://github.com/DataDog/glommio)
- [smol](https://github.com/smol-rs/smol)
- [tokio](https://github.com/tokio-rs/tokio)

## Usage

```rust
use deboa::{
    HttpClient,
    request::{DeboaRequest, FetchWith, get},
    Result,
};
use deboa_tokio::Client;
use deboa_extras::serde::json::JsonBody;

#[tokio::main]
async fn main() -> Result<()> {
  // Create a new Client instance, set timeouts, catches and protocol.
  let client = Client::new();

  let posts: Vec<Post> = get("https://jsonplaceholder.typicode.com/posts")?
    .header(header::CONTENT_TYPE, "application/json")
    .send_with(&client)
    .await?
    .body_as(JsonBody)
    .await?;

  println!("posts: {:#?}", posts);

  Ok(())
}
```

## Subprojects

### [deboa](https://github.com/deboa-client/deboa/tree/develop/deboa)

Top level project with runtime agnostic http client API.

### [deboa-macros](https://github.com/deboa-client/deboa/tree/develop/deboa-macros)

A crate with collection of convenience macros for deboa. It is close equivalent to
apisauce for axios, where one macro does it all, from request to response.
It used to be the home of bora macro, which has been moved to vamo-macros crate.

### deboa-compio (moved)

Deboa implementation for compio runtime to <https://github.com/deboa-client/deboa-compio>.

### deboa-glommio (moved)

Deboa implementation for glommio runtime to <https://github.com/deboa-client/deboa-glommio>.

### deboa-smol (moved)

Deboa implementation for smol runtime to <https://github.com/deboa-client/deboa/deboa-smol>.

### deboa-tokio (moved)

Deboa implmentation for tokio runtime to <https://github.com/deboa-client/deboa/deboa-tokio>.

### vamo (moved)

Vamo has moved to <https://github.com/deboa-client/vamo>.

### vamo-macros (moved)

Vamo-macros has moved to <https://github.com/deboa-client/vamo>.

## License

Licensed under either of

- Apache License, Version 2.0
  (LICENSE-APACHE or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license
  (LICENSE-MIT or <https://opensource.org/licenses/MIT>)

at your option.

## Author

Rogerio Pereira Araujo <rogerio.araujo@gmail.com>
