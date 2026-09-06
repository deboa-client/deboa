//! Test utilities and modules

use ::url::Url;
use http::Uri;

mod cache;
mod cookie;
mod form;
mod request;
mod response;
mod url;

const TEST_URL: &str = "https://localhost:8000";

pub(crate) fn test_url() -> Url {
    Url::parse(TEST_URL).unwrap()
}

pub(crate) fn test_uri() -> Uri {
    Uri::from_static(TEST_URL)
}
