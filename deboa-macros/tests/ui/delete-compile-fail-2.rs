use deboa::TestResult;
use deboa_macros::delete;
use deboa_tokio::Client;

#[tokio::main]
async fn main() -> TestResult<()> {
    let client = Client::default();
    delete!(
        client => &client,
    );
    Ok(())
}
