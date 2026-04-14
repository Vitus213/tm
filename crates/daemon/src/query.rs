use std::collections::BTreeMap;

use chrono::{DateTime, Datelike, TimeZone, Timelike, Utc};
use tm_core::{ActivityKind, ClosedSession};
use tm_ipc::{
    ActivityFilter, ChartBucket, ChartsQuery, ChartsResponse, OverviewQuery, OverviewResponse,
    SessionRow, SessionsQuery, SessionsResponse, SummaryBucket, TimeRange, TrendPoint,
};
use tm_storage::RepositoryError;

use crate::SessionRepository;

pub struct QueryService<R> {
    repo: R,
}

impl<R> QueryService<R>
where
    R: SessionRepository,
{
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn get_overview(
        &self,
        query: OverviewQuery,
    ) -> Result<OverviewResponse, RepositoryError> {
        let sessions = scoped_sessions(
            self.repo.list_sessions().await?,
            &query.range,
            ActivityFilter::All,
            None,
        );

        // Compute once and derive truncated lists to avoid redundant sorting
        let all_apps = top_buckets(&sessions, ActivityKind::App, 20);
        let all_websites = top_buckets(&sessions, ActivityKind::Website, 20);

        let top_apps = all_apps.iter().take(5).cloned().collect();
        let top_websites = all_websites.iter().take(5).cloned().collect();

        Ok(OverviewResponse {
            range: query.range,
            total_seconds: sessions.iter().map(|row| row.duration_seconds).sum(),
            top_apps,
            top_websites,
            more_apps: all_apps,
            more_websites: all_websites,
            recent_sessions: recent_rows(&sessions),
        })
    }

    pub async fn get_sessions(
        &self,
        query: SessionsQuery,
    ) -> Result<SessionsResponse, RepositoryError> {
        let items = scoped_sessions(
            self.repo.list_sessions().await?,
            &query.range,
            query.activity_filter,
            query.subject_query.as_deref(),
        );
        Ok(SessionsResponse {
            range: query.range,
            activity_filter: query.activity_filter,
            subject_query: query.subject_query,
            items,
        })
    }

    pub async fn get_charts(&self, query: ChartsQuery) -> Result<ChartsResponse, RepositoryError> {
        let sessions = scoped_sessions(
            self.repo.list_sessions().await?,
            &query.range,
            ActivityFilter::All,
            None,
        );
        Ok(ChartsResponse {
            range: query.range,
            app_share: top_buckets(&sessions, ActivityKind::App, 10),
            website_share: top_buckets(&sessions, ActivityKind::Website, 10),
            daily_totals: daily_totals(&sessions),
            hourly_totals: hourly_totals(&sessions),
        })
    }

    pub fn repo(&self) -> &R {
        &self.repo
    }
}

fn scoped_sessions(
    sessions: Vec<ClosedSession>,
    range: &TimeRange,
    filter: ActivityFilter,
    subject_query: Option<&str>,
) -> Vec<SessionRow> {
    let query = subject_query.map(|value| value.to_ascii_lowercase());
    let mut rows = sessions
        .into_iter()
        .filter(|session| {
            session.ended_at() > range.started_at && session.started_at() < range.ended_at
        })
        .filter(|session| match filter {
            ActivityFilter::All => true,
            ActivityFilter::App => session.kind() == ActivityKind::App,
            ActivityFilter::Website => session.kind() == ActivityKind::Website,
        })
        .filter_map(|session| row_for_range(session, range))
        .filter(|row| {
            query
                .as_ref()
                .map(|query| row.subject_id.to_ascii_lowercase().contains(query))
                .unwrap_or(true)
        })
        .collect::<Vec<_>>();

    rows.sort_by_key(|row| std::cmp::Reverse(row.started_at));
    rows
}

fn row_for_range(session: ClosedSession, range: &TimeRange) -> Option<SessionRow> {
    let started_at = session.started_at().max(range.started_at);
    let ended_at = session.ended_at().min(range.ended_at);
    let duration_seconds = (ended_at - started_at).num_seconds();

    (duration_seconds > 0).then_some(SessionRow {
        kind: session.kind(),
        subject_id: session.subject_id().to_owned(),
        title: session.title().to_owned(),
        started_at,
        ended_at,
        duration_seconds,
    })
}

fn top_buckets(rows: &[SessionRow], kind: ActivityKind, limit: usize) -> Vec<SummaryBucket> {
    let mut grouped: BTreeMap<String, SummaryBucket> = BTreeMap::new();
    for row in rows.iter().filter(|row| row.kind == kind) {
        let entry = grouped
            .entry(row.subject_id.clone())
            .or_insert_with(|| SummaryBucket {
                kind,
                subject_id: row.subject_id.clone(),
                title: row.title.clone(),
                total_seconds: 0,
            });
        entry.total_seconds += row.duration_seconds;
        entry.title = row.title.clone();
    }

    let mut buckets = grouped.into_values().collect::<Vec<_>>();
    buckets.sort_by_key(|bucket| std::cmp::Reverse(bucket.total_seconds));
    buckets.truncate(limit);
    buckets
}

fn recent_rows(rows: &[SessionRow]) -> Vec<SessionRow> {
    rows.iter().take(10).cloned().collect()
}

fn daily_totals(rows: &[SessionRow]) -> Vec<TrendPoint> {
    let mut grouped: BTreeMap<DateTime<Utc>, i64> = BTreeMap::new();
    for row in rows {
        let day = Utc
            .with_ymd_and_hms(
                row.started_at.year(),
                row.started_at.month(),
                row.started_at.day(),
                0,
                0,
                0,
            )
            .unwrap();
        *grouped.entry(day).or_default() += row.duration_seconds;
    }

    grouped
        .into_iter()
        .map(|(bucket_start, total_seconds)| TrendPoint {
            bucket_start,
            total_seconds,
        })
        .collect()
}

fn hourly_totals(rows: &[SessionRow]) -> Vec<ChartBucket> {
    let mut grouped: BTreeMap<u32, i64> = BTreeMap::new();
    for row in rows {
        *grouped.entry(row.started_at.hour()).or_default() += row.duration_seconds;
    }

    grouped
        .into_iter()
        .map(|(hour, total_seconds)| ChartBucket {
            label: format!("{hour:02}:00"),
            total_seconds,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_row(kind: ActivityKind, subject_id: &str, duration_seconds: i64) -> SessionRow {
        let started_at = Utc::now();
        SessionRow {
            kind,
            subject_id: subject_id.to_string(),
            title: format!("Title {subject_id}"),
            started_at,
            ended_at: started_at + chrono::Duration::seconds(duration_seconds),
            duration_seconds,
        }
    }

    #[test]
    fn top_buckets_respects_limit() {
        let mut rows = Vec::new();
        // Create 25 app sessions with descending durations so they have a deterministic order
        for i in 0..25 {
            rows.push(make_row(ActivityKind::App, &format!("app-{i:02}"), 1000 - i as i64));
        }
        // Add some website sessions to ensure filtering by kind works
        for i in 0..10 {
            rows.push(make_row(ActivityKind::Website, &format!("site-{i:02}"), 500 - i as i64));
        }

        let apps_5 = top_buckets(&rows, ActivityKind::App, 5);
        assert_eq!(apps_5.len(), 5);
        assert_eq!(apps_5[0].subject_id, "app-00");
        assert_eq!(apps_5[4].subject_id, "app-04");

        let apps_20 = top_buckets(&rows, ActivityKind::App, 20);
        assert_eq!(apps_20.len(), 20);
        assert_eq!(apps_20[0].subject_id, "app-00");
        assert_eq!(apps_20[19].subject_id, "app-19");

        let sites_5 = top_buckets(&rows, ActivityKind::Website, 5);
        assert_eq!(sites_5.len(), 5);
        assert_eq!(sites_5[0].subject_id, "site-00");
        assert_eq!(sites_5[4].subject_id, "site-04");

        let sites_20 = top_buckets(&rows, ActivityKind::Website, 20);
        assert_eq!(sites_20.len(), 10); // only 10 websites exist
    }
}
