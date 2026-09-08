use deboa::TestResult;
use deboa_macros::patch;
use deboa_tokio::Client;

#[tokio::main]
async fn main() -> TestResult<()> {
    let client = Client::default();
    patch!(
        client => &client,
    );
    Ok(())
}
