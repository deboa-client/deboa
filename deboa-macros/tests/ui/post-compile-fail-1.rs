use deboa::TestResult;
use deboa_macros::post;
use deboa_tokio::Client;

#[tokio::main]
async fn main() -> TestResult<()> {
    let client = Client::default();
    post!(
        url => "https://jsonplaceholder.typicode.com/posts",
    );
    Ok(())
}
