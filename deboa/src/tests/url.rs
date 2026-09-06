use crate::url::IntoUrl;
use caramelo::{expect, matchers::ok};

#[test]
fn test_url() {
    let url_str = "http://example.com";
    let url = url_str
        .parse_url()
        .unwrap();
    assert_eq!(url.scheme(), "http");
    assert_eq!(url.host_str(), Some("example.com"));
}

#[test]
fn test_url_invalid() {
    let url_str = "invalid_url";
    let url = url_str.parse_url();
    assert!(url.is_err());
}

#[test]
fn test_url_string_ref() {
    let url_str = &String::from("http://example.com");
    let url = url_str.into_url();
    expect(url).to_be(ok());
}

#[test]
fn test_url_string() {
    let url_str = &mut String::from("http://example.com");
    let url = url_str.into_url();
    expect(url).to_be(ok());
}
