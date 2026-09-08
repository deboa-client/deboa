use deboa::TestResult;
use deboa_macros::submit;
use deboa_tokio::Client;

#[tokio::main]
async fn main() -> TestResult<()> {
    let client = Client::default();
    submit!(
        client => &client,
        method => http::Method::GET,
    );
    Ok(())
}
