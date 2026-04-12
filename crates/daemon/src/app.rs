use std::{path::PathBuf, time::Duration};

use anyhow::Result;
use chrono::{DateTime, Utc};
use tm_storage::SqliteRepository;
use tm_tracker::{
    FocusedWindowSnapshot, focused_window_once, map_snapshot_to_event, should_emit_focus_event,
};

use crate::session_service::{SessionRepository, SessionService};

const POLL_INTERVAL: Duration = Duration::from_secs(1);

fn default_db_path() -> PathBuf {
    db_path_from_env(
        std::env::var_os("XDG_DATA_HOME").map(PathBuf::from),
        std::env::var_os("HOME").map(PathBuf::from),
    )
}

fn db_path_from_env(xdg_data_home: Option<PathBuf>, home: Option<PathBuf>) -> PathBuf {
    xdg_data_home
        .unwrap_or_else(|| {
            let mut path = home.expect("HOME must be set");
            path.push(".local");
            path.push("share");
            path
        })
        .join("tm")
        .join("tm.db")
}

pub async fn run() -> Result<()> {
    let repo = SqliteRepository::open_at(default_db_path()).await?;
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
    match focused_window_once() {
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
            *previous_focus = None;
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
    use chrono::{TimeZone, Utc};
    use tm_storage::SqliteRepository;
    use tm_tracker::FocusedWindowSnapshot;

    use crate::SessionService;

    #[test]
    fn db_path_uses_xdg_data_home_when_present() {
        let path = db_path_from_env(
            Some("/tmp/tm-xdg-test".into()),
            Some("/tmp/tm-home-test".into()),
        );

        assert_eq!(path, PathBuf::from("/tmp/tm-xdg-test/tm/tm.db"));
    }

    #[test]
    fn db_path_falls_back_to_home_local_share() {
        let path = db_path_from_env(None, Some("/tmp/tm-home-test".into()));

        assert_eq!(
            path,
            PathBuf::from("/tmp/tm-home-test/.local/share/tm/tm.db")
        );
    }

    #[tokio::test]
    async fn focus_loss_clears_cache_so_same_window_can_emit_again() {
        let repo = SqliteRepository::in_memory().await.unwrap();
        let mut service = SessionService::new(repo);
        let mut previous_focus = None;

        apply_focus_snapshot(
            &mut service,
            &mut previous_focus,
            Some(sample_snapshot(7, Some("firefox"), "Rust docs", 9, 0, 0)),
        )
        .await
        .unwrap();
        apply_focus_snapshot(&mut service, &mut previous_focus, None)
            .await
            .unwrap();
        apply_focus_snapshot(
            &mut service,
            &mut previous_focus,
            Some(sample_snapshot(7, Some("firefox"), "Rust docs", 9, 5, 0)),
        )
        .await
        .unwrap();
        service
            .flush(Utc.with_ymd_and_hms(2026, 4, 12, 9, 10, 0).unwrap())
            .await
            .unwrap();

        let sessions = service.list_sessions().await.unwrap();

        assert_eq!(sessions.len(), 2);
        assert_eq!(
            sessions[0].started_at(),
            Utc.with_ymd_and_hms(2026, 4, 12, 9, 0, 0).unwrap()
        );
        assert_eq!(
            sessions[0].ended_at(),
            Utc.with_ymd_and_hms(2026, 4, 12, 9, 5, 0).unwrap()
        );
        assert_eq!(
            sessions[1].started_at(),
            Utc.with_ymd_and_hms(2026, 4, 12, 9, 5, 0).unwrap()
        );
        assert_eq!(
            sessions[1].ended_at(),
            Utc.with_ymd_and_hms(2026, 4, 12, 9, 10, 0).unwrap()
        );
    }

    fn sample_snapshot(
        window_id: u64,
        app_id: Option<&str>,
        title: &str,
        hour: u32,
        minute: u32,
        second: u32,
    ) -> FocusedWindowSnapshot {
        FocusedWindowSnapshot {
            window_id,
            app_id: app_id.map(str::to_owned),
            title: title.to_owned(),
            pid: Some(4242),
            observed_at: Utc
                .with_ymd_and_hms(2026, 4, 12, hour, minute, second)
                .unwrap(),
        }
    }
}
