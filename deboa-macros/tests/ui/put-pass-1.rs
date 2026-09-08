use deboa::TestResult;
use deboa_macros::put;
use deboa_tokio::Client;

#[tokio::main]
async fn main() -> TestResult<()> {
    let client = Client::default();
    put!(
        url => "https://jsonplaceholder.typicode.com/posts",
        client => &client,
    );
    Ok(())
}
