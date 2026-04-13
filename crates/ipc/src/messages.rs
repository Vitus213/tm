use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tm_core::ActivityKind;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeRange {
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityFilter {
    All,
    App,
    Website,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OverviewQuery {
    pub range: TimeRange,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionsQuery {
    pub range: TimeRange,
    pub activity_filter: ActivityFilter,
    pub subject_query: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChartsQuery {
    pub range: TimeRange,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SummaryBucket {
    pub kind: ActivityKind,
    pub subject_id: String,
    pub title: String,
    pub total_seconds: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionRow {
    pub kind: ActivityKind,
    pub subject_id: String,
    pub title: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub duration_seconds: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrendPoint {
    pub bucket_start: DateTime<Utc>,
    pub total_seconds: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChartBucket {
    pub label: String,
    pub total_seconds: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OverviewResponse {
    pub range: TimeRange,
    pub total_seconds: i64,
    pub top_apps: Vec<SummaryBucket>,
    pub top_websites: Vec<SummaryBucket>,
    pub recent_sessions: Vec<SessionRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionsResponse {
    pub range: TimeRange,
    pub activity_filter: ActivityFilter,
    pub subject_query: Option<String>,
    pub items: Vec<SessionRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChartsResponse {
    pub range: TimeRange,
    pub app_share: Vec<SummaryBucket>,
    pub website_share: Vec<SummaryBucket>,
    pub daily_totals: Vec<TrendPoint>,
    pub hourly_totals: Vec<ChartBucket>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DaemonCommand {
    FlushActiveSession,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DaemonEvent {
    Ack,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DaemonRequest {
    Ping,
    GetOverview(OverviewQuery),
    GetSessions(SessionsQuery),
    GetCharts(ChartsQuery),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DaemonResponse {
    Pong,
    Overview(OverviewResponse),
    Sessions(SessionsResponse),
    Charts(ChartsResponse),
    Error { message: String },
}
