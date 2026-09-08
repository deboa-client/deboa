use deboa::TestResult;
use deboa_macros::get;
use deboa_tokio::Client;

#[tokio::main]
async fn main() -> TestResult<()> {
    let client = Client::default();
    get!(
        url => "https://jsonplaceholder.typicode.com/posts",
    );
    Ok(())
}
