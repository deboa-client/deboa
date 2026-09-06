use crate::{
    cookie::DeboaCookie,
    response::{DeboaResponse, IntoBody},
    TestResult,
};
use http::{header, Response};
use hyper_body_utils::HttpBody;

const SAMPLE_TEST: &[u8] = b"Hello, world!";

#[test]
fn test_status() -> TestResult<()> {
    let response = Response::builder()
        .status(http::StatusCode::OK)
        .body(SAMPLE_TEST.into_body())
        .unwrap();

    let response = DeboaResponse::new(response);
    assert_eq!(response.status(), http::StatusCode::OK);
    Ok(())
}

#[test]
fn test_headers() -> TestResult<()> {
    let response = DeboaResponse::builder()
        .status(http::StatusCode::OK)
        .headers(http::HeaderMap::new())
        .empty();
    assert_eq!(*response.headers(), http::HeaderMap::new());
    Ok(())
}

#[test]
fn test_cookies() -> TestResult<()> {
    let mut headers = http::HeaderMap::new();
    headers.insert(header::SET_COOKIE, http::HeaderValue::from_static("test=test"));
    let response = DeboaResponse::builder()
        .status(http::StatusCode::OK)
        .headers(headers)
        .build();
    assert_eq!(response.cookies(), Ok(Some(vec![DeboaCookie::new("test", "test")])));
    Ok(())
}

#[test]
fn test_header() -> TestResult<()> {
    let response = DeboaResponse::builder()
        .status(http::StatusCode::OK)
        .header(header::ACCEPT_LANGUAGE, "pt-BR")
        .build();
    assert_eq!(
        response
            .headers()
            .get(header::ACCEPT_LANGUAGE),
        Some(&http::HeaderValue::from_static("pt-BR"))
    );
    Ok(())
}

#[test]
fn test_content_type() -> TestResult<()> {
    let response = DeboaResponse::builder()
        .status(http::StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html")
        .build();
    assert_eq!(response.content_type(), Ok("text/html".to_string()));
    Ok(())
}

#[test]
fn test_content_length() -> TestResult<()> {
    let response = DeboaResponse::builder()
        .status(http::StatusCode::OK)
        .header(header::CONTENT_LENGTH, "9")
        .build();
    assert_eq!(response.content_length(), Ok(9));
    Ok(())
}

#[test]
fn test_to_bytes() -> TestResult<()> {
    let response = Response::builder()
        .status(http::StatusCode::OK)
        .body(SAMPLE_TEST.into_body())
        .unwrap();

    let response = DeboaResponse::new(response);
    let body = match response.inner_body() {
        HttpBody::Standard(bytes) => bytes,
        _ => panic!("Expected standard body"),
    };
    assert_eq!(body.into_inner(), Some(SAMPLE_TEST.into()));
    Ok(())
}

#[test]
fn test_body() -> TestResult<()> {
    let response = DeboaResponse::builder()
        .body(SAMPLE_TEST)
        .build();
    let body = match response.inner_body() {
        HttpBody::Standard(bytes) => bytes,
        _ => panic!("Expected standard body"),
    };
    assert_eq!(body.into_inner(), Some(SAMPLE_TEST.into()));
    Ok(())
}

#[test]
fn test_version_mut() -> TestResult<()> {
    let mut response = DeboaResponse::builder()
        .version(http::Version::HTTP_11)
        .build();
    *response.version_mut() = http::Version::HTTP_2;
    assert_eq!(response.version(), http::Version::HTTP_2);
    Ok(())
}

#[test]
fn test_status_mut() -> TestResult<()> {
    let mut response = DeboaResponse::builder()
        .status(http::StatusCode::OK)
        .build();
    *response.status_mut() = http::StatusCode::NOT_FOUND;
    assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
    Ok(())
}

#[test]
fn test_headers_mut() -> TestResult<()> {
    let mut response = DeboaResponse::builder()
        .status(http::StatusCode::OK)
        .build();
    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, http::HeaderValue::from_static("text/html"));
    assert_eq!(
        response
            .headers()
            .get(header::CONTENT_TYPE),
        Some(&http::HeaderValue::from_static("text/html"))
    );
    Ok(())
}

#[test]
fn test_into_parts() -> TestResult<()> {
    let response = DeboaResponse::builder()
        .status(http::StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html")
        .body(SAMPLE_TEST)
        .build();
    let (parts, body) = response.into_parts();
    assert_eq!(parts.status, http::StatusCode::OK);
    assert_eq!(
        parts
            .headers
            .get(header::CONTENT_TYPE),
        Some(&http::HeaderValue::from_static("text/html"))
    );
    let body = match body {
        HttpBody::Standard(bytes) => bytes,
        _ => panic!("Expected standard body"),
    };
    assert_eq!(body.into_inner(), Some(SAMPLE_TEST.into()));
    Ok(())
}

#[test]
fn test_debug() -> TestResult<()> {
    let response = DeboaResponse::builder()
        .status(http::StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html")
        .version(http::Version::HTTP_2)
        .body(SAMPLE_TEST)
        .build();
    let debug_str = format!("{:?}", response);
    assert!(debug_str.contains("DeboaResponse"));
    assert!(debug_str.contains("status: 200"));
    assert!(debug_str.contains("headers: {\"content-type\": \"text/html\"}"));
    assert!(debug_str.contains("version: HTTP/2"));
    Ok(())
}
