use deboa::TestResult;
use deboa_macros::patch;
use deboa_tokio::Client;

#[tokio::main]
async fn main() -> TestResult<()> {
    let client = Client::default();
    patch!(
        url => "https://jsonplaceholder.typicode.com/posts",
        client => &client,
    );
    Ok(())
}
