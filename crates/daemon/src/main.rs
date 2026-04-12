#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tm_daemon::run().await
}
