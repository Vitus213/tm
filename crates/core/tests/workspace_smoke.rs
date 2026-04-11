use tm_core::workspace_ready;

#[test]
fn workspace_exposes_core_crate() {
    assert_eq!(workspace_ready(), "tm-core-ready");
}
