use crate::{
    form::{DeboaForm, EncodedForm, MultiPartForm},
    Result, TestResult,
};
use std::{io::Write, path::Path};
use tempfile::NamedTempFile;

#[test]
fn test_encoded_form() -> Result<()> {
    let form = EncodedForm::builder()
        .field("name", "deboa")
        .field("version", "0.0.1");

    let form = form.build();

    assert_eq!(form.to_vec(), b"name=deboa&version=0.0.1");

    Ok(())
}

#[test]
fn test_multipart_form() -> Result<()> {
    let builder = MultiPartForm::builder()
        .field("name", "deboa")
        .field("version", "0.0.1");

    let boundary = builder.boundary();

    let form = builder.build();

    assert_eq!(form.to_vec(), format!("--{}\r\nContent-Disposition: form-data; name=\"name\"\r\n\r\ndeboa\r\n--{}\r\nContent-Disposition: form-data; name=\"version\"\r\n\r\n0.0.1\r\n--{}--\r\n", boundary, boundary, boundary).as_bytes());

    Ok(())
}

#[test]
fn test_encoded_form_content_type() -> Result<()> {
    let form = EncodedForm::builder()
        .field("name", "deboa")
        .field("version", "0.0.1");

    assert_eq!(form.content_type(), "application/x-www-form-urlencoded");

    Ok(())
}

#[test]
fn test_multipart_form_content_type() -> Result<()> {
    let form = MultiPartForm::builder()
        .field("name", "deboa")
        .field("version", "0.0.1");

    assert_eq!(
        form.content_type(),
        "multipart/form-data; boundary=".to_string() + &form.boundary()
    );

    Ok(())
}

#[test]
fn test_multipart_form_with_file() -> TestResult<()> {
    let mut builder = MultiPartForm::builder();

    let mut tmpfile = NamedTempFile::new()?;
    write!(tmpfile, "Hello World!")?;
    tmpfile.persist("/tmp/test.txt")?;

    let path = Path::new("/tmp/test.txt");
    builder = builder
        .field("name", "deboa")
        .field("version", "0.0.1")
        .file("file", &path);

    let boundary = builder.boundary();
    let form = builder.build();

    let mut chunk = format!("--{}\r\nContent-Disposition: form-data; name=\"name\"\r\n\r\ndeboa\r\n--{}\r\nContent-Disposition: form-data; name=\"version\"\r\n\r\n0.0.1\r\n--{}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"test.txt\"\r\nContent-Type: text/plain\r\n\r\n", boundary, boundary, boundary).as_bytes().to_vec();
    chunk.extend_from_slice(b"Hello World!");
    chunk.extend_from_slice(format!("\r\n--{}--\r\n", boundary).as_bytes());

    assert_eq!(form.to_vec(), chunk);

    Ok(())
}
