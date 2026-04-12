use std::{path::PathBuf, time::Duration};

use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, Utc};
use tm_storage::SqliteRepository;
use tm_tracker::{
    FocusedWindowSnapshot, focused_window_once, map_snapshot_to_event, should_emit_focus_event,
};

use crate::session_service::{SessionRepository, SessionService};

const POLL_INTERVAL: Duration = Duration::from_secs(1);

pub(crate) fn default_db_path() -> Result<PathBuf> {
    db_path_from_env(
        std::env::var_os("XDG_DATA_HOME").map(PathBuf::from),
        std::env::var_os("HOME").map(PathBuf::from),
    )
}

pub(crate) fn db_path_from_env(
    xdg_data_home: Option<PathBuf>,
    home: Option<PathBuf>,
) -> Result<PathBuf> {
    let data_dir = match xdg_data_home {
        Some(path) => path,
        None => {
            let mut path =
                home.ok_or_else(|| anyhow!("HOME is not set; cannot resolve database path"))?;
            path.push(".local");
            path.push("share");
            path
        }
    };

    Ok(data_dir.join("tm").join("tm.db"))
}

pub async fn run() -> Result<()> {
    let db_path = default_db_path().context("failed to resolve daemon database path")?;
    let repo = SqliteRepository::open_at(db_path).await?;
    let mut service = SessionService::new(repo);
    let mut previous_focus = None;
    let mut interval = tokio::time::interval(POLL_INTERVAL);

    loop {
        tokio::select! {
            result = tokio::signal::ctrl_c() => {
                result?;
                flush_active_session(&mut service, Utc::now()).await?;
                return Ok(());
            }
            _ = interval.tick() => {
                poll_tracker_once(&mut service, &mut previous_focus).await?;
            }
        }
    }
}

async fn poll_tracker_once<R>(
    service: &mut SessionService<R>,
    previous_focus: &mut Option<FocusedWindowSnapshot>,
) -> Result<()>
where
    R: SessionRepository,
{
    match tokio::task::spawn_blocking(focused_window_once).await? {
        Ok(snapshot) => apply_focus_snapshot(service, previous_focus, snapshot).await?,
        Err(error) => {
            eprintln!("tm-daemon: tracker poll failed: {error}");
        }
    }

    Ok(())
}

async fn apply_focus_snapshot<R>(
    service: &mut SessionService<R>,
    previous_focus: &mut Option<FocusedWindowSnapshot>,
    snapshot: Option<FocusedWindowSnapshot>,
) -> Result<()>
where
    R: SessionRepository,
{
    match snapshot {
        Some(current) => {
            if should_emit_focus_event(previous_focus.as_ref(), &current) {
                service.ingest(map_snapshot_to_event(&current)).await?;
            }
            *previous_focus = Some(current);
        }
        None => {
            if previous_focus.take().is_some() {
                flush_active_session(service, Utc::now()).await?;
            }
        }
    }

    Ok(())
}

async fn flush_active_session<R>(
    service: &mut SessionService<R>,
    ended_at: DateTime<Utc>,
) -> Result<()>
where
    R: SessionRepository,
{
    service.flush(ended_at).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{apply_focus_snapshot, db_path_from_env};
    use chrono::{Duration, Utc};
    use tm_storage::SqliteRepository;
    use tm_tracker::FocusedWindowSnapshot;

    use crate::SessionService;

    #[test]
    fn db_path_uses_xdg_data_home_when_present() {
        let path = db_path_from_env(
            Some("/tmp/tm-xdg-test".into()),
            Some("/tmp/tm-home-test".into()),
        )
        .unwrap();

        assert_eq!(path, PathBuf::from("/tmp/tm-xdg-test/tm/tm.db"));
    }

    #[test]
    fn db_path_falls_back_to_home_local_share() {
        let path = db_path_from_env(None, Some("/tmp/tm-home-test".into())).unwrap();

        assert_eq!(
            path,
            PathBuf::from("/tmp/tm-home-test/.local/share/tm/tm.db")
        );
    }

    #[test]
    fn db_path_missing_home_returns_error_instead_of_panicking() {
        let error = db_path_from_env(None, None).expect_err("missing HOME should return an error");
        assert!(
            error.to_string().contains("HOME is not set"),
            "unexpected error: {error:#}"
        );
    }

    #[tokio::test]
    async fn focus_loss_closes_active_session_at_gap_start_and_same_window_can_emit_again() {
        let repo = SqliteRepository::in_memory().await.unwrap();
        let mut service = SessionService::new(repo);
        let mut previous_focus = None;
        let started_at = Utc::now() - Duration::minutes(10);

        apply_focus_snapshot(
            &mut service,
            &mut previous_focus,
            Some(sample_snapshot(7, Some("firefox"), "Rust docs", started_at)),
        )
        .await
        .unwrap();

        let no_focus_before = Utc::now();
        apply_focus_snapshot(&mut service, &mut previous_focus, None)
            .await
            .unwrap();
        let no_focus_after = Utc::now();

        let resumed_at = no_focus_after + Duration::minutes(5);
        apply_focus_snapshot(
            &mut service,
            &mut previous_focus,
            Some(sample_snapshot(7, Some("firefox"), "Rust docs", resumed_at)),
        )
        .await
        .unwrap();

        let flushed_at = resumed_at + Duration::minutes(5);
        service.flush(flushed_at).await.unwrap();

        let sessions = service.list_sessions().await.unwrap();

        assert_eq!(sessions.len(), 2);
        assert_eq!(sessions[0].started_at(), started_at);
        assert!(sessions[0].ended_at() >= no_focus_before);
        assert!(sessions[0].ended_at() <= no_focus_after);
        assert_eq!(sessions[1].started_at(), resumed_at);
        assert_eq!(sessions[1].ended_at(), flushed_at);
    }

    fn sample_snapshot(
        window_id: u64,
        app_id: Option<&str>,
        title: &str,
        observed_at: chrono::DateTime<Utc>,
    ) -> FocusedWindowSnapshot {
        FocusedWindowSnapshot {
            window_id,
            app_id: app_id.map(str::to_owned),
            title: title.to_owned(),
            pid: Some(4242),
            observed_at,
        }
    }
}
