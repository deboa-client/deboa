use deboa::TestResult;
use deboa_macros::submit;
use deboa_tokio::Client;

#[tokio::main]
async fn main() -> TestResult<()> {
    let client = Client::default();
    submit!(
        method => http::Method::GET,
        url => "https://jsonplaceholder.typicode.com/posts",
        client => &client,
    );
    Ok(())
}
