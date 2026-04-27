use tm_storage::SqliteRepository;

#[tokio::test]
async fn reads_default_settings_when_none_stored() {
    let repo = SqliteRepository::in_memory().await.unwrap();

    let settings = repo.get_settings().await.unwrap();

    assert_eq!(settings.idle_threshold_seconds, 300);
    assert!(settings.website_tracking_enabled);
    assert!(settings.autostart_enabled);
}

#[tokio::test]
async fn stores_and_reads_back_settings() {
    let repo = SqliteRepository::in_memory().await.unwrap();

    let updated = tm_storage::Settings {
        idle_threshold_seconds: 600,
        website_tracking_enabled: false,
        autostart_enabled: false,
    };

    repo.save_settings(&updated).await.unwrap();

    let settings = repo.get_settings().await.unwrap();
    assert_eq!(settings.idle_threshold_seconds, 600);
    assert!(!settings.website_tracking_enabled);
    assert!(!settings.autostart_enabled);
}
