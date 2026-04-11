use tm_storage::SqliteRepository;

use crate::session_service::SessionService;

pub async fn run() -> anyhow::Result<()> {
    eprintln!(
        "tm-daemon is using in-memory session storage; tracked sessions will be lost on shutdown."
    );

    let repo = SqliteRepository::in_memory().await?;
    let _service = SessionService::new(repo);

    tokio::signal::ctrl_c().await?;
    Ok(())
}
