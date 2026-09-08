use deboa::TestResult;
use deboa_macros::stream;
use deboa_tokio::Client;

#[tokio::main]
async fn main() -> TestResult<()> {
    let client = Client::default();
    stream!(
        url => "https://jsonplaceholder.typicode.com/posts",
    );
    Ok(())
}
