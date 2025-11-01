use reauth_core::run;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run().await
}
