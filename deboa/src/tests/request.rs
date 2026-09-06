use crate::{
    form::{DeboaForm, EncodedForm},
    request::{delete, get, patch, post, put, query, DeboaRequest, IntoRequest, MethodExt},
    tests::{test_uri, test_url, TEST_URL},
    TestResult,
};
use bytes::Bytes;
use caramelo::{
    expect,
    matchers::{custom, eq},
};
use http::{header, HeaderValue, Method, Uri, Version};
use hyper_body_utils::HttpBody;
use std::str::FromStr;

#[test]
fn test_method_ext_from_url() -> TestResult<()> {
    let methods =
        vec![Method::GET, Method::POST, Method::QUERY, Method::PUT, Method::PATCH, Method::DELETE];
    for method in &methods {
        let request = method
            .clone()
            .from_url(TEST_URL)?
            .build()?;
        expect(request.method()).to_be(eq(method));
        expect(request.uri()).to_be(eq(&test_uri()));
    }
    Ok(())
}

#[test]
#[should_panic = "Method not supported"]
fn test_method_ext_from_url_failure() {
    Method::OPTIONS
        .from_url(TEST_URL)
        .unwrap()
        .build()
        .unwrap();
}

#[test]
#[should_panic = "Method not supported"]
fn test_method_ext_from_url_str_failure() {
    "OPTIONS"
        .from_url(TEST_URL)
        .unwrap()
        .build()
        .unwrap();
}

#[test]
fn test_method_ext_to_url() -> TestResult<()> {
    let request = Method::POST
        .to_url(TEST_URL)?
        .build()?;
    expect(request.method()).to_be(eq(&Method::POST));
    expect(request.uri()).to_be(eq(&test_uri()));
    Ok(())
}

#[test]
fn test_str_method_ext_from_url() -> TestResult<()> {
    let methods = vec!["GET", "POST", "QUERY", "PATCH", "PUT", "DELETE"];
    for method in methods {
        let request = method
            .from_url(TEST_URL)?
            .build()?;
        expect(request.method()).to_be(eq(&Method::from_str(method).unwrap()));
        expect(request.uri()).to_be(eq(&test_uri()));
    }
    Ok(())
}

#[test]
fn test_str_method_ext_to_url() -> TestResult<()> {
    let request = "POST"
        .to_url(TEST_URL)?
        .build()?;
    assert_eq!(request.method(), &Method::POST);
    assert_eq!(*request.uri(), test_uri());
    Ok(())
}

#[test]
fn test_into_url() -> TestResult<()> {
    let url = test_url();
    let request = DeboaRequest::get(url)?.build()?;
    expect(request.uri()).to_be(eq(&test_uri()));
    Ok(())
}

#[test]
fn test_into_request_from_str() -> TestResult<()> {
    let url = test_url();
    let request = url
        .clone()
        .into_request()?;
    expect(request.uri()).to_be(eq(&test_uri()));
    Ok(())
}

#[test]
fn test_into_request_from_str_slice() -> TestResult<()> {
    let url = "http://example.com";
    let request = url.into_request()?;
    expect(
        request
            .uri()
            .to_string(),
    )
    .to_be(eq("http://example.com/"));
    Ok(())
}

#[test]
fn test_into_request_from_string() -> TestResult<()> {
    let url = test_url();
    let post_url = format!("{}posts/{}", &url, 1);
    let request = post_url
        .clone()
        .into_request()?;
    let uri = Uri::from_str(
        url.join("/posts/1")?
            .as_ref(),
    )?;
    expect(request.uri()).to_be(eq(&uri));
    Ok(())
}

#[test]
fn test_into_str() -> TestResult<()> {
    let url = test_url();
    let request = DeboaRequest::get(url.clone())?.build()?;
    expect(request.uri()).to_be(eq(&test_uri()));
    Ok(())
}

#[test]
fn test_into_string() -> TestResult<()> {
    let url = test_url();
    let request = DeboaRequest::get(url.clone())?.build()?;
    expect(request.uri()).to_be(eq(&test_uri()));
    Ok(())
}

#[test]
fn test_from_str_method_and_url() -> TestResult<()> {
    let request = DeboaRequest::from_str(
        r##"
    GET https://localhost:8000
    "##,
    )?;
    expect(request.method()).to_be(eq(&Method::GET));
    expect(request.uri()).to_be(eq(&Uri::from_static("https://localhost:8000")));
    Ok(())
}

#[test]
fn test_from_str_headers() -> TestResult<()> {
    let request = DeboaRequest::from_str(
        r##"
    GET https://localhost:8000
    Content-Type: application/json
    "##,
    )?;
    expect(
        request
            .headers()
            .get(header::CONTENT_TYPE),
    )
    .to_be(eq(Some(&HeaderValue::from_str("application/json").unwrap())));
    Ok(())
}

#[test]
fn test_base_url() -> TestResult<()> {
    let url = test_url();
    let api = DeboaRequest::get(url.clone())?.build()?;
    assert_eq!(*api.uri(), test_uri());
    Ok(())
}

#[test]
fn test_set_headers() -> TestResult<()> {
    let url = test_url();
    let request = DeboaRequest::get(url)?
        .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())?
        .build()?;

    expect(
        request
            .headers()
            .get(&header::CONTENT_TYPE),
    )
    .to_be(eq(Some(&HeaderValue::from_str(mime::APPLICATION_JSON.as_ref()).unwrap())));

    Ok(())
}

#[test]
fn test_set_headers_as_tuple() -> TestResult<()> {
    let headers = vec![(header::CONTENT_TYPE, mime::APPLICATION_JSON.to_string())];
    let request = DeboaRequest::get(test_url())?
        .headers(headers)
        .build()?;

    expect(
        request
            .headers()
            .get(&header::CONTENT_TYPE),
    )
    .to_be(eq(Some(&HeaderValue::from_str(mime::APPLICATION_JSON.as_ref()).unwrap())));

    Ok(())
}

#[test]
fn test_set_headers_as_str_tuple() -> TestResult<()> {
    let headers = vec![("Content-Type", "application/json")];
    let request = DeboaRequest::get(test_url())?
        .headers(headers)
        .build()?;

    expect(
        request
            .headers()
            .get(&header::CONTENT_TYPE),
    )
    .to_be(eq(Some(&HeaderValue::from_str(mime::APPLICATION_JSON.as_ref()).unwrap())));

    Ok(())
}

#[test]
fn test_set_headers_as_string_tuple() -> TestResult<()> {
    let headers = vec![("Content-Type".to_owned(), "application/json".to_owned())];
    let request = DeboaRequest::get(test_url())?
        .headers(headers)
        .build()?;

    expect(
        request
            .headers()
            .get(&header::CONTENT_TYPE),
    )
    .to_be(eq(Some(&HeaderValue::from_str(mime::APPLICATION_JSON.as_ref()).unwrap())));

    Ok(())
}

#[test]
fn test_set_basic_auth() -> TestResult<()> {
    let url = test_url();
    let request = DeboaRequest::get(url)?
        .basic_auth("username", "password")?
        .build()?;

    expect(
        request
            .headers()
            .get(&header::AUTHORIZATION),
    )
    .to_be(eq(Some(&HeaderValue::from_str("Basic dXNlcm5hbWU6cGFzc3dvcmQ=").unwrap())));

    Ok(())
}

#[test]
fn test_set_bearer_auth() -> TestResult<()> {
    let url = test_url();
    let request = DeboaRequest::get(url)?
        .bearer_auth("token")?
        .build()?;

    expect(
        request
            .headers()
            .get(&header::AUTHORIZATION),
    )
    .to_be(eq(Some(&HeaderValue::from_str("Bearer token").unwrap())));

    Ok(())
}

#[test]
fn test_add_header() -> TestResult<()> {
    let url = test_url();
    let request = DeboaRequest::get(url)?
        .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())?
        .build()?;

    expect(
        request
            .headers()
            .get(&header::CONTENT_TYPE),
    )
    .to_be(eq(Some(&HeaderValue::from_str(mime::APPLICATION_JSON.as_ref()).unwrap())));

    Ok(())
}

#[test]
fn test_put() -> TestResult<()> {
    let url = test_url();
    let api = DeboaRequest::put(url.clone())?.build()?;
    assert_eq!(*api.method(), Method::PUT);
    Ok(())
}

#[test]
fn test_patch() -> TestResult<()> {
    let url = test_url();
    let api = DeboaRequest::patch(url.clone())?.build()?;
    assert_eq!(*api.method(), Method::PATCH);
    Ok(())
}

#[test]
fn test_query() -> TestResult<()> {
    let url = test_url();
    let api = DeboaRequest::query(url.clone())?.build()?;
    assert_eq!(*api.method(), Method::QUERY);
    Ok(())
}

#[test]
fn test_delete() -> TestResult<()> {
    let url = test_url();
    let api = DeboaRequest::delete(url.clone())?.build()?;
    assert_eq!(*api.method(), Method::DELETE);
    Ok(())
}

#[test]
fn test_version() -> TestResult<()> {
    let url = test_url();
    let api = DeboaRequest::delete(url.clone())?.build()?;
    assert_eq!(api.version(), Version::HTTP_2);
    Ok(())
}

#[test]
fn test_set_version() -> TestResult<()> {
    let url = test_url();
    let api = DeboaRequest::delete(url.clone())?
        .version(Version::HTTP_11)
        .build()?;
    assert_eq!(api.version(), Version::HTTP_11);
    Ok(())
}

#[test]
fn test_set_text() -> TestResult<()> {
    let url = test_url();
    let api = DeboaRequest::delete(url.clone())?
        .text("Some text")
        .build()?;
    let body = api
        .body()
        .into_body();
    let text = match body {
        HttpBody::Standard(text) => text,
        _ => panic!("Expected standard body"),
    };
    assert_eq!(text.into_inner(), Some(Bytes::from("Some text")));
    Ok(())
}

#[test]
fn test_quick_get() -> TestResult<()> {
    let url = test_url();
    let api = get(url.clone())?.build()?;
    assert_eq!(*api.method(), Method::GET);
    Ok(())
}

#[test]
fn test_quick_put() -> TestResult<()> {
    let url = test_url();
    let api = put(url.clone())?.build()?;
    assert_eq!(*api.method(), Method::PUT);
    Ok(())
}

#[test]
fn test_quick_patch() -> TestResult<()> {
    let url = test_url();
    let api = patch(url.clone())?.build()?;
    assert_eq!(*api.method(), Method::PATCH);
    Ok(())
}

#[test]
fn test_quick_query() -> TestResult<()> {
    let url = test_url();
    let api = query(url.clone())?.build()?;
    assert_eq!(*api.method(), Method::QUERY);
    Ok(())
}

#[test]
fn test_quick_delete() -> TestResult<()> {
    let url = test_url();
    let api = delete(url.clone())?.build()?;
    assert_eq!(*api.method(), Method::DELETE);
    Ok(())
}

#[test]
fn test_request_body() -> TestResult<()> {
    let url = test_url();
    let api = delete(url.clone())?
        .body("Text".into())
        .build()?;
    let body = api
        .body()
        .into_body();
    let bytes = match body {
        HttpBody::Standard(bytes) => bytes,
        _ => panic!("Expected standard body"),
    };

    assert_eq!(
        bytes
            .into_inner()
            .unwrap_or_default(),
        Bytes::from("Text")
    );

    Ok(())
}

#[test]
fn test_request_body_bytes() -> TestResult<()> {
    let url = test_url();
    let api = delete(url.clone())?
        .bytes(b"Text")
        .build()?;
    let body = api
        .body()
        .into_body();
    let bytes = match body {
        HttpBody::Standard(bytes) => bytes,
        _ => panic!("Expected standard body"),
    };

    assert_eq!(
        bytes
            .into_inner()
            .unwrap_or_default(),
        Bytes::from("Text")
    );

    Ok(())
}

#[test]
fn test_request_form() -> TestResult<()> {
    let form = EncodedForm::builder()
        .field("name", "deboa")
        .field("version", "0.0.1");
    let url = test_url();
    let api = post(url.clone())?
        .form(form.into())?
        .build()?;
    let body = api
        .body()
        .into_body();
    let bytes = match body {
        HttpBody::Standard(bytes) => bytes,
        _ => panic!("Expected standard body"),
    };

    assert_eq!(
        bytes
            .into_inner()
            .unwrap_or_default(),
        Bytes::from(b"name=deboa&version=0.0.1" as &[u8])
    );

    Ok(())
}

#[test]
fn test_request_from_parts() -> TestResult<()> {
    let (parts, _) = http::Request::builder()
        .method(Method::POST)
        .uri(test_url().as_str())
        .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
        .body(())?
        .into_parts();

    let request = DeboaRequest::from_parts(parts, "Some text".into())?;

    expect(request.method()).to_be(eq(&Method::POST));

    let (my_parts, my_body) = request.into_parts();
    expect(my_parts.method).to_be(eq(Method::POST));
    expect(my_body).to_have(custom(
        |body| match body {
            HttpBody::Standard(bytes)
                if bytes
                    .clone()
                    .into_inner()
                    == Some(Bytes::from("Some text")) =>
            {
                true
            }
            _ => false,
        },
        "custom matcher for HttpBody",
    ));

    Ok(())
}
