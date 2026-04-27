use tm_daemon::QueryService;
use tm_ipc::Settings;
use tm_storage::SqliteRepository;

#[tokio::test]
async fn returns_default_settings_from_database() {
    let repo = SqliteRepository::in_memory().await.unwrap();
    let service = QueryService::new(repo);

    let settings = service.get_settings().await.unwrap();

    assert_eq!(settings.idle_threshold_seconds, 300);
    assert!(settings.website_tracking_enabled);
    assert!(settings.autostart_enabled);
}

#[tokio::test]
async fn update_settings_persists_changes() {
    let repo = SqliteRepository::in_memory().await.unwrap();
    let service = QueryService::new(repo);

    let updated = Settings {
        idle_threshold_seconds: 600,
        website_tracking_enabled: false,
        autostart_enabled: false,
    };

    service.update_settings(updated).await.unwrap();

    let settings = service.get_settings().await.unwrap();
    assert_eq!(settings.idle_threshold_seconds, 600);
    assert!(!settings.website_tracking_enabled);
    assert!(!settings.autostart_enabled);
}
