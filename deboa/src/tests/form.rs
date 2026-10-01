use crate::{
    form::{EncodedForm, MultiPartForm},
    TestResult,
};
use bytes::BytesMut;
use futures::StreamExt;
use std::{fmt::Write as _, io::Write};
use tempfile::NamedTempFile;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

#[test]
fn test_encoded_form() -> TestResult<()> {
    let form = EncodedForm::builder()
        .field("name", "deboa")
        .field("version", "0.0.1");

    let form = form.build();

    assert_eq!(form.to_vec(), b"name=deboa&version=0.0.1");

    Ok(())
}

#[tokio::test]
async fn test_multipart_form() -> TestResult<()> {
    let builder = MultiPartForm::builder()
        .field("name", "deboa")
        .field("version", "0.0.1");

    let boundary = builder.boundary();

    let mut data = BytesMut::new();
    write!(&mut data, "--{}\r\nContent-Disposition: form-data; name=\"name\"\r\n\r\ndeboa\r\n--{}\r\nContent-Disposition: form-data; name=\"version\"\r\n\r\n0.0.1\r\n--{}--\r\n", boundary, boundary, boundary)?;

    let form = builder
        .build()
        .await
        .fold(Vec::new(), |mut acc, chunk| async move {
            acc.extend_from_slice(
                chunk
                    .unwrap()
                    .data_ref()
                    .unwrap(),
            );
            acc
        })
        .await;
    assert_eq!(&form, &data);

    Ok(())
}

#[test]
fn test_encoded_form_content_type() -> TestResult<()> {
    let form = EncodedForm::builder()
        .field("name", "deboa")
        .field("version", "0.0.1");

    assert_eq!(form.content_type(), "application/x-www-form-urlencoded");

    Ok(())
}

#[test]
fn test_multipart_form_content_type() -> TestResult<()> {
    let form = MultiPartForm::builder()
        .field("name", "deboa")
        .field("version", "0.0.1");

    assert_eq!(
        form.content_type(),
        "multipart/form-data; boundary=".to_string() + &form.boundary()
    );

    Ok(())
}

#[tokio::test]
async fn test_multipart_form_with_file() -> TestResult<()> {
    let mut builder = MultiPartForm::builder();

    let mut tmpfile = NamedTempFile::new()?;
    write!(tmpfile, "Hello World!")?;

    let file_stream = ReaderStream::new(File::open(tmpfile.path()).await?);
    builder = builder
        .field("name", "deboa")
        .field("version", "0.0.1")
        .stream("file", "test.txt", "text/plain", file_stream);

    let boundary = builder.boundary();
    let mut bytes = BytesMut::new();
    write!(&mut bytes, "--{}\r\nContent-Disposition: form-data; name=\"name\"\r\n\r\ndeboa\r\n--{}\r\nContent-Disposition: form-data; name=\"version\"\r\n\r\n0.0.1\r\n--{}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"test.txt\"\r\nContent-Type: text/plain\r\n\r\nHello World!\r\n--{}--\r\n", boundary, boundary, boundary, boundary)?;

    let form = builder
        .build()
        .await
        .fold(Vec::new(), |mut acc, chunk| async move {
            acc.extend_from_slice(
                chunk
                    .unwrap()
                    .data_ref()
                    .unwrap(),
            );
            acc
        })
        .await;
    assert_eq!(&form, &bytes);

    Ok(())
}
