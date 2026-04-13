mod messages;
mod socket;

pub use messages::{
    ActivityFilter, ChartBucket, ChartsQuery, ChartsResponse, DaemonCommand, DaemonEvent,
    DaemonRequest, DaemonResponse, OverviewQuery, OverviewResponse, SessionRow, SessionsQuery,
    SessionsResponse, SummaryBucket, TimeRange, TrendPoint,
};
pub use socket::{default_socket_path, socket_path_from_env};

pub fn ipc_ready() -> &'static str {
    "tm-ipc-ready"
}
