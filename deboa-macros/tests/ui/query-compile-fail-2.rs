use deboa::TestResult;
use deboa_macros::query;
use deboa_tokio::Client;

#[tokio::main]
async fn main() -> TestResult<()> {
    let client = Client::default();
    query!(
        client => &client,
    );
    Ok(())
}
