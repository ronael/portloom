use portloom::infrastructure::system::context::discover;
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};
#[test]
fn project_discovery_is_bounded_and_skips_dependency_directories() {
    let dir = std::env::temp_dir().join(format!(
        "portloom-project-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(dir.join("src/deep")).unwrap();
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = Cleanup(dir.clone());
    fs::write(dir.join("package.json"), r#"{"name":"my project"}"#).unwrap();
    assert_eq!(
        discover(dir.join("src/deep").to_str().unwrap(), None)
            .unwrap()
            .name,
        "my project"
    );
    assert!(discover(dir.join("node_modules/lib").to_str().unwrap(), None).is_none());
    assert!(discover(dir.to_str().unwrap(), dir.to_str()).is_none());
    assert!(discover("relative", None).is_none());
    fs::write(dir.join("package.json"), "broken").unwrap();
    fs::write(dir.join("Cargo.toml"), "").unwrap();
    assert!(discover(dir.to_str().unwrap(), None).is_some());
}
