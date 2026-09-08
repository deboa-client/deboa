use deboa::TestResult;
use deboa_macros::query;
use deboa_tokio::Client;

#[tokio::main]
async fn main() -> TestResult<()> {
    let client = Client::default();
    query!(
        url => "https://jsonplaceholder.typicode.com/posts",
        client => &client,
    );
    Ok(())
}
