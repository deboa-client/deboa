use deboa::TestResult;
use deboa_macros::submit;
use deboa_tokio::Client;

#[tokio::main]
async fn main() -> TestResult<()> {
    let client = Client::default();
    submit!(
        url => "https://jsonplaceholder.typicode.com/posts",
        method => http::Method::GET,
    );
    Ok(())
}
