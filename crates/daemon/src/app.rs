use tm_storage::SqliteRepository;

use crate::session_service::SessionService;

pub async fn run() -> anyhow::Result<()> {
    let repo = SqliteRepository::in_memory().await?;
    let _service = SessionService::new(repo);
    Ok(())
}
