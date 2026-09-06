use crate::{cookie::DeboaCookie, TestResult};
use caramelo::{
    expect,
    matchers::{eq, truthy},
};
use cookie::{Cookie, Expiration};
use time::OffsetDateTime;

#[test]
fn test_new_cookie() {
    let cookie = DeboaCookie::new("test", "test");

    expect(cookie.name()).to_be(eq("test"));
    expect(cookie.value()).to_be(eq("test"));
}

#[test]
fn test_set_expires() {
    let mut cookie = DeboaCookie::new("test", "test");

    let now = OffsetDateTime::now_utc();
    cookie.set_expires(Expiration::from(now));

    expect(
        cookie
            .expires()
            .unwrap()
            .datetime(),
    )
    .to_be(eq(Some(now)));
}

#[test]
fn test_set_path() {
    let mut cookie = DeboaCookie::new("test", "test");

    cookie.set_path("/test");

    expect(
        cookie
            .path()
            .unwrap(),
    )
    .to_be(eq("/test"));
}

#[test]
fn test_set_domain() {
    let mut cookie = DeboaCookie::new("test", "test");

    cookie.set_domain("test.com");

    expect(
        cookie
            .domain()
            .unwrap(),
    )
    .to_be(eq("test.com"));
}

#[test]
fn test_set_secure() {
    let mut cookie = DeboaCookie::new("test", "test");

    cookie.set_secure(true);

    assert!(cookie
        .secure()
        .unwrap());
}

#[test]
fn test_set_http_only() {
    let mut cookie = DeboaCookie::new("test", "test");

    cookie.set_http_only(true);

    expect(
        cookie
            .http_only()
            .unwrap(),
    )
    .to_be(truthy());
}

#[test]
fn test_parse_from_header() -> TestResult<()> {
    let cookie = DeboaCookie::parse_from_header("test=test")?;

    expect(cookie.name()).to_be(eq("test"));
    expect(cookie.value()).to_be(eq("test"));

    Ok(())
}

#[test]
fn test_display() -> TestResult<()> {
    let cookie = DeboaCookie::parse_from_header("test=test")?;
    expect(format!("{cookie}")).to_be(eq("test=test"));
    Ok(())
}

#[test]
fn test_debug() -> TestResult<()> {
    let cookie = DeboaCookie::parse_from_header("test=test")?;
    expect(format!("{:?}", cookie)).to_be(eq("DeboaCookie { name: \"test\", value: \"test\", expires: None, path: None, domain: None, secure: None, http_only: None }"));
    Ok(())
}

#[test]
fn test_from_deboacookie() -> TestResult<()> {
    let mut deboka_cookie = DeboaCookie::parse_from_header("test=test")?;
    deboka_cookie.set_domain("example.com");
    deboka_cookie.set_http_only(true);
    deboka_cookie.set_path("/this");
    deboka_cookie.set_secure(true);
    deboka_cookie.set_expires(Expiration::from(OffsetDateTime::now_utc()));
    let http_cookie = Cookie::from(deboka_cookie);
    expect(http_cookie.value()).to_be(eq("test"));
    Ok(())
}
