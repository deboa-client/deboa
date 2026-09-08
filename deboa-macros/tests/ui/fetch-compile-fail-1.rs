use deboa::TestResult;
use deboa_macros::fetch;
use deboa_tokio::Client;

#[tokio::main]
async fn main() -> TestResult<()> {
    let client = Client::default();
    fetch!(
        url => "https://jsonplaceholder.typicode.com/posts",
    );
    Ok(())
}
