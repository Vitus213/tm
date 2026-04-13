use chrono::{TimeZone, Utc};
use tm_core::{ActivityKind, ClosedSession};
use tm_daemon::QueryService;
use tm_ipc::{ActivityFilter, ChartsQuery, OverviewQuery, SessionsQuery, TimeRange};
use tm_storage::SqliteRepository;

#[tokio::test]
async fn overview_splits_app_and_website_rankings() {
    let repo = SqliteRepository::in_memory().await.unwrap();
    seed(
        &repo,
        &[
            session(ActivityKind::App, "wezterm", "shell", 9, 0, 9, 10),
            session(ActivityKind::Website, "docs.rs", "Rust docs", 9, 10, 9, 25),
            session(ActivityKind::App, "firefox", "ChatGPT", 9, 25, 9, 40),
        ],
    )
    .await;

    let service = QueryService::new(repo);
    let result = service
        .get_overview(OverviewQuery { range: day_range() })
        .await
        .unwrap();

    assert_eq!(result.total_seconds, 2_400);
    assert_eq!(result.top_apps.len(), 2);
    assert_eq!(result.top_apps[0].subject_id, "firefox");
    assert_eq!(result.top_websites.len(), 1);
    assert_eq!(result.top_websites[0].subject_id, "docs.rs");
    assert_eq!(result.recent_sessions[0].subject_id, "firefox");
}

#[tokio::test]
async fn sessions_query_filters_by_kind_and_subject() {
    let repo = SqliteRepository::in_memory().await.unwrap();
    seed(
        &repo,
        &[
            session(ActivityKind::App, "wezterm", "shell", 9, 0, 9, 10),
            session(ActivityKind::Website, "docs.rs", "Rust docs", 9, 10, 9, 25),
            session(
                ActivityKind::Website,
                "news.ycombinator.com",
                "HN",
                9,
                25,
                9,
                35,
            ),
        ],
    )
    .await;

    let service = QueryService::new(repo);
    let result = service
        .get_sessions(SessionsQuery {
            range: day_range(),
            activity_filter: ActivityFilter::Website,
            subject_query: Some("docs".into()),
        })
        .await
        .unwrap();

    assert_eq!(result.items.len(), 1);
    assert_eq!(result.items[0].subject_id, "docs.rs");
}

#[tokio::test]
async fn charts_query_returns_daily_and_hourly_series() {
    let repo = SqliteRepository::in_memory().await.unwrap();
    seed(
        &repo,
        &[
            session(ActivityKind::App, "wezterm", "shell", 9, 0, 9, 30),
            session(ActivityKind::Website, "docs.rs", "Rust docs", 10, 0, 10, 15),
        ],
    )
    .await;

    let service = QueryService::new(repo);
    let result = service
        .get_charts(ChartsQuery { range: day_range() })
        .await
        .unwrap();

    assert_eq!(result.app_share[0].subject_id, "wezterm");
    assert_eq!(result.website_share[0].subject_id, "docs.rs");
    assert_eq!(result.daily_totals.len(), 1);
    assert_eq!(result.hourly_totals.len(), 2);
}

fn day_range() -> TimeRange {
    TimeRange {
        started_at: Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap(),
        ended_at: Utc.with_ymd_and_hms(2026, 4, 13, 23, 59, 59).unwrap(),
    }
}

fn session(
    kind: ActivityKind,
    subject_id: &str,
    title: &str,
    start_hour: u32,
    start_minute: u32,
    end_hour: u32,
    end_minute: u32,
) -> ClosedSession {
    ClosedSession::new(
        kind,
        subject_id.into(),
        title.into(),
        Utc.with_ymd_and_hms(2026, 4, 13, start_hour, start_minute, 0)
            .unwrap(),
        Utc.with_ymd_and_hms(2026, 4, 13, end_hour, end_minute, 0)
            .unwrap(),
    )
    .unwrap()
}

async fn seed(repo: &SqliteRepository, sessions: &[ClosedSession]) {
    for session in sessions {
        repo.insert_session(session).await.unwrap();
    }
}
